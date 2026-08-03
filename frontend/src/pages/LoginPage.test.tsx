import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { MemoryRouter } from "react-router-dom"
import { beforeEach, describe, expect, it, vi } from "vitest"
import { ApiError } from "@/lib/api"
import LoginPage from "@/pages/LoginPage"

const { login, toastError } = vi.hoisted(() => ({
  login: vi.fn<(password: string) => Promise<void>>(),
  toastError: vi.fn(),
}))

vi.mock("@/providers/auth-provider", () => ({
  useAuth: () => ({ isAuthenticated: false, login }),
}))

vi.mock("sonner", () => ({
  toast: { error: toastError },
}))

function renderLogin() {
  const queryClient = new QueryClient({
    defaultOptions: { mutations: { retry: false } },
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>
        <LoginPage />
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

beforeEach(() => {
  login.mockReset()
  toastError.mockReset()
})

describe("LoginPage", () => {
  it("announces empty-password validation and focuses the field", async () => {
    const user = userEvent.setup()
    renderLogin()

    await user.click(screen.getByRole("button", { name: "进入控制台" }))

    const password = screen.getByLabelText("访问密码")
    const error = screen.getByRole("alert")
    expect(error).toHaveTextContent("请输入访问密码")
    expect(password).toHaveFocus()
    expect(password).toHaveAttribute("aria-invalid", "true")
    expect(password).toHaveAttribute("aria-describedby", error.id)
    expect(toastError).toHaveBeenCalledWith("请输入访问密码")
    expect(login).not.toHaveBeenCalled()
  })

  it("localizes invalid-password errors without exposing server details", async () => {
    const user = userEvent.setup()
    login.mockRejectedValue(new ApiError("invalid credentials: auth hash mismatch", 401, "unauthorized"))
    renderLogin()

    const password = screen.getByLabelText("访问密码")
    await user.type(password, "wrong-password")
    await user.click(screen.getByRole("button", { name: "进入控制台" }))

    const error = await screen.findByRole("alert")
    expect(error).toHaveTextContent("访问密码不正确，请重试")
    expect(error).not.toHaveTextContent("auth hash mismatch")
    await waitFor(() => expect(password).toHaveFocus())
  })

  it("uses a generic localized message for unexpected login failures", async () => {
    const user = userEvent.setup()
    login.mockRejectedValue(new Error("database host unavailable at 10.0.0.4"))
    renderLogin()

    const password = screen.getByLabelText("访问密码")
    await user.type(password, "secret")
    await user.click(screen.getByRole("button", { name: "进入控制台" }))

    const error = await screen.findByRole("alert")
    expect(error).toHaveTextContent("登录失败，请稍后重试")
    expect(error).not.toHaveTextContent("10.0.0.4")
    await waitFor(() => expect(password).toHaveFocus())
  })

  it("provides a 44px visibility target and toggles password visibility", async () => {
    const user = userEvent.setup()
    renderLogin()

    const password = screen.getByLabelText("访问密码")
    const toggle = screen.getByRole("button", { name: "显示密码" })
    expect(toggle).toHaveClass("size-11")
    expect(password).toHaveAttribute("type", "password")

    await user.click(toggle)

    expect(password).toHaveAttribute("type", "text")
    expect(screen.getByRole("button", { name: "隐藏密码" })).toHaveClass("size-11")
  })
})
