import { AlertTriangle, RefreshCw } from "lucide-react"
import { ApiError } from "@/lib/api"
import { Button } from "@/components/ui/button"

function queryErrorDetails(error: unknown): { message: string; hint: string } {
  if (error instanceof ApiError) {
    if (error.status === 401) {
      return { message: "登录状态已失效", hint: "请重新登录后继续。" }
    }
    if (error.status === 403) {
      return { message: "当前账号无权执行此操作", hint: "请检查服务端授权策略。" }
    }
    if (error.status >= 500) {
      return { message: error.message, hint: "服务暂时不可用，请重试；持续失败时检查服务日志。" }
    }
    return { message: error.message, hint: `请求未完成（HTTP ${error.status}）。` }
  }

  if (error instanceof TypeError) {
    return { message: "无法连接管理服务", hint: "请检查网络连接和服务健康状态。" }
  }

  if (error instanceof Error && error.message.trim()) {
    return { message: error.message, hint: "请重试；持续失败时检查服务日志。" }
  }

  return { message: "请求未完成", hint: "请重试；持续失败时检查服务日志。" }
}

export function QueryErrorState({
  error,
  onRetry,
  title = "加载失败",
}: Readonly<{
  error: unknown
  onRetry: () => void
  title?: string
}>) {
  const details = queryErrorDetails(error)

  return (
    <div role="alert" className="flex flex-col items-center gap-3 py-24 text-center">
      <AlertTriangle className="size-7 text-danger" aria-hidden="true" />
      <div>
        <p className="text-sm font-medium text-danger">{title}</p>
        <p className="mt-1 max-w-lg font-mono text-xs text-text-secondary">{details.message}</p>
        <p className="mt-1 max-w-lg text-xs text-text-muted">{details.hint}</p>
      </div>
      <Button type="button" variant="outline" onClick={onRetry}>
        <RefreshCw />
        重试
      </Button>
    </div>
  )
}
