# AIPocket 昨晚复盘 + 修复记录 (2026-08-04)

## 用户问题：一晚上了有实际收获吗？
- results 表仅 1 条 valid：OpenRouter key（sk-or-v1-63b...fb1, $0.5 余额, GitHub 泄漏, run_2026_08_03_08-40-57 旧版手动轮）
- 昨晚 8 轮自动扫描（14:54-21:54，共 151 候选：41+40+11+5+23+4+20+7）results 全空
- 结论：**不是 bug** —— results 表只写 valid/suspicious（scanner.rs ~713 insert_results），昨晚通过率 0 与 08:40 轮 1/510=0.2% 量级一致；泄漏 key 大多已被 revoke，真实场景
- 教训：candidates 多 ≠ 收获多；多源发现 2709 原始命中 → 2272 目标 → 仅个位数候选有余额

## 严重问题：调度器静默死亡 9h
- 现象：22:08:02 最后一轮 Finished 后，22:54 起再无 Started，API 一直正常（18006 端口 404 快速响应），CPU 0%，无 panic 无信号，容器 healthy
- 根因链：scan 完成日志后执行 lease.release().await（scan_lock.rs:81）→ redis Script invoke_async **无超时** → 连接坏死时永久挂起 → job() 永不返回 → run_with_interval 外层 select 卡死（job 分支不 poll tick）→ 调度永久停止；run_watch 错误被 `let _ =` 吞掉
- 证据：docker logs --since 10h 有 4176 行但最后时间戳全是 22:08（说明 22:08 后没有任何新日志）；--tail 8 最后一行 "run_2026_08_03_21-54-04"（scan completed 日志的尾部，release 前）

## 三层修复（已实施，待测试+部署）
1. **scheduler.rs** run_with_interval：job() 包 `tokio::time::timeout(period*4)`，超时 abort + warn + Err → backoff 重试（之前 multiplier=2 会误杀 65ms/20ms 测试 job，改 4 后 80ms>65ms 通过；真实 scan 最长 63min < 1h*4）
2. **scan_lock.rs** release：redis invoke_async 包 10s 超时（`anyhow!` import 已补）
3. **main.rs** serve：scheduler spawn 改 watchdog loop —— run_watch 退出后 warn + sleep 30s + 重启（原 `let _ = run_watch(...)` 静默吞错）

## 待办
- [x] cargo test -p aipocket-services -p aipocket-db -p aipocket（修 multiplier 后需重跑）
- [ ] 构建 + 部署 T6（docker compose）
- [ ] 验证：重启后立即扫描一轮 + 调度继续 ticking
- [ ] git push（含 Co-Authored-By）
- [ ] 汇报用户

## 环境事实
- API 端口：127.0.0.1:18006（容器内 8000），/healthz 返回 404（无此路由，服务活着）
- cargo 不在非交互 PATH：需 `export PATH="$HOME/.cargo/bin:$PATH"`
- scan lock：redis key aipocket:scan:lock，heartbeat 续期，release 无超时（已修）
- shodan key 已失效（401）——提醒用户换
- run_watch cancel 仅由 SIGINT/SIGTERM 触发（main.rs:202-205）；调度无外部健康检查——watchdog 是唯一保障
