import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { MemoryRouter } from "react-router-dom"
import { describe, expect, it, vi } from "vitest"
import { ApiError, api } from "@/lib/api"
import { QueryErrorState } from "@/components/query-error-state"
import HistoryPage from "@/pages/HistoryPage"

describe("QueryErrorState", () => {
  it("shows an actionable server failure and retries", async () => {
    const user = userEvent.setup()
    const onRetry = vi.fn()

    render(
      <QueryErrorState
        error={new ApiError("database unavailable", 503, "service_unavailable")}
        onRetry={onRetry}
      />,
    )

    expect(screen.getByRole("alert")).toHaveTextContent("database unavailable")
    expect(screen.getByRole("alert")).toHaveTextContent("检查服务日志")
    await user.click(screen.getByRole("button", { name: "重试" }))
    expect(onRetry).toHaveBeenCalledOnce()
  })

  it("distinguishes a network failure", () => {
    render(<QueryErrorState error={new TypeError("Failed to fetch")} onRetry={vi.fn()} />)

    expect(screen.getByRole("alert")).toHaveTextContent("无法连接管理服务")
    expect(screen.getByRole("alert")).toHaveTextContent("请检查网络连接和服务健康状态")
  })

  it("recovers the history list after a 503 and an explicit retry", async () => {
    const user = userEvent.setup()
    const getRuns = vi
      .spyOn(api, "getRuns")
      .mockRejectedValueOnce(new ApiError("database unavailable", 503, "service_unavailable"))
      .mockResolvedValueOnce({
        days: [
          {
            day: "2026-08-03",
            runs: [
              {
                run_id: "run-recovered",
                started_at: "2026-08-03T09:30:00Z",
                valid_count: 2,
                suspicious_count: 1,
                has_log: false,
                raw_hits: 4,
                unique_targets: 3,
                candidates: 3,
                active_requests: 3,
                final_verified: 2,
                suspicious: 1,
                sources: ["github"],
                scan_mode: "full",
                high_value_final: 0,
                deletable: false,
              },
            ],
          },
        ],
      })
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    })

    render(
      <QueryClientProvider client={queryClient}>
        <MemoryRouter>
          <HistoryPage />
        </MemoryRouter>
      </QueryClientProvider>,
    )

    expect(await screen.findByRole("alert")).toHaveTextContent("扫描记录加载失败")
    expect(screen.getByRole("alert")).toHaveTextContent("database unavailable")
    await user.click(screen.getByRole("button", { name: "重试" }))

    expect(await screen.findByText("run-recovered")).toBeInTheDocument()
    expect(screen.queryByRole("alert")).not.toBeInTheDocument()
    expect(getRuns).toHaveBeenCalledTimes(2)

    getRuns.mockRestore()
    queryClient.clear()
  })
})
