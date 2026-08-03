use aipocket_core::Settings;
use anyhow::Result;
use std::{future::Future, sync::Arc, time::Duration};
use tokio::time::{self, MissedTickBehavior};
use tokio_util::sync::CancellationToken;

const MAX_FAILURE_BACKOFF_MULTIPLIER: u32 = 32;

#[derive(Clone)]
pub struct Scheduler {
    settings: Arc<Settings>,
}

impl Scheduler {
    pub fn new(settings: Arc<Settings>) -> Self {
        Self { settings }
    }

    pub async fn run_forever<F, Fut>(&self, cancel: CancellationToken, job: F) -> Result<()>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<()>>,
    {
        self.run_with_interval(
            cancel,
            Duration::from_secs(self.settings.scheduler_interval.max(1)),
            job,
        )
        .await
    }

    async fn run_with_interval<F, Fut>(
        &self,
        cancel: CancellationToken,
        period: Duration,
        mut job: F,
    ) -> Result<()>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<()>>,
    {
        let mut interval = time::interval(period);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let max_failure_backoff = period.saturating_mul(MAX_FAILURE_BACKOFF_MULTIPLIER);
        let mut failure_backoff = period;

        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = interval.tick() => {
                    let result = tokio::select! {
                        _ = cancel.cancelled() => break,
                        result = job() => result,
                    };

                    match result {
                        Ok(()) => failure_backoff = period,
                        Err(error) => {
                            tracing::warn!(?error, ?failure_backoff, "scheduled job failed; retrying");
                            tokio::select! {
                                _ = cancel.cancelled() => break,
                                _ = time::sleep(failure_backoff) => {}
                            }
                            failure_backoff = failure_backoff
                                .saturating_mul(2)
                                .min(max_failure_backoff);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::{Mutex, Notify};

    fn scheduler() -> Scheduler {
        Scheduler::new(Arc::new(Settings::default()))
    }

    #[tokio::test]
    async fn job_error_is_isolated_and_next_run_can_succeed() {
        let cancel = CancellationToken::new();
        let stop = cancel.clone();
        let attempts = Arc::new(AtomicUsize::new(0));
        let seen = attempts.clone();

        scheduler()
            .run_with_interval(cancel, Duration::from_millis(2), move || {
                let attempt = seen.fetch_add(1, Ordering::SeqCst);
                let stop = stop.clone();
                async move {
                    if attempt == 0 {
                        Err(anyhow!("fixture failure"))
                    } else {
                        stop.cancel();
                        Ok(())
                    }
                }
            })
            .await
            .unwrap();

        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn consecutive_failures_use_bounded_exponential_backoff() {
        let period = Duration::from_millis(2);
        let cancel = CancellationToken::new();
        let stop = cancel.clone();
        let attempts = Arc::new(AtomicUsize::new(0));
        let seen = attempts.clone();
        let started = time::Instant::now();

        scheduler()
            .run_with_interval(cancel, period, move || {
                let attempt = seen.fetch_add(1, Ordering::SeqCst);
                let stop = stop.clone();
                async move {
                    if attempt == 6 {
                        stop.cancel();
                        Ok(())
                    } else {
                        Err(anyhow!("fixture failure"))
                    }
                }
            })
            .await
            .unwrap();

        let minimum_backoff = period.saturating_mul(1 + 2 + 4 + 8 + 16 + 32);
        assert!(started.elapsed() >= minimum_backoff);
        assert_eq!(attempts.load(Ordering::SeqCst), 7);
    }

    #[tokio::test]
    async fn long_jobs_do_not_overlap_or_trigger_burst_catch_up() {
        let period = Duration::from_millis(20);
        let cancel = CancellationToken::new();
        let stop = cancel.clone();
        let starts = Arc::new(Mutex::new(Vec::new()));
        let seen = starts.clone();

        scheduler()
            .run_with_interval(cancel, period, move || {
                let seen = seen.clone();
                let stop = stop.clone();
                async move {
                    let mut starts = seen.lock().await;
                    starts.push(time::Instant::now());
                    let count = starts.len();
                    drop(starts);
                    match count {
                        1 => time::sleep(Duration::from_millis(65)).await,
                        3 => stop.cancel(),
                        _ => {}
                    }
                    Ok(())
                }
            })
            .await
            .unwrap();

        let starts = starts.lock().await;
        assert_eq!(starts.len(), 3);
        assert!(starts[1] - starts[0] >= Duration::from_millis(65));
        assert!(starts[2] - starts[1] >= Duration::from_millis(10));
    }

    #[tokio::test]
    async fn cancellation_interrupts_a_running_job() {
        let cancel = CancellationToken::new();
        let stop = cancel.clone();
        let entered = Arc::new(Notify::new());
        let observed = entered.clone();

        let task = tokio::spawn(async move {
            scheduler()
                .run_with_interval(cancel, Duration::from_secs(60), move || {
                    let observed = observed.clone();
                    async move {
                        observed.notify_one();
                        std::future::pending().await
                    }
                })
                .await
                .unwrap();
        });

        entered.notified().await;
        stop.cancel();
        time::timeout(Duration::from_millis(100), task)
            .await
            .expect("scheduler did not stop after cancellation")
            .unwrap();
    }
}
