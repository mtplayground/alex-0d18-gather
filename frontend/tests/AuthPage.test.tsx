import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import AuthPage from "../src/pages/AuthPage";
import { startRegistration } from "../src/lib/authApi";

const authState = vi.hoisted(() => ({
  status: "unauthenticated",
  user: null,
}));

vi.mock("../src/auth/useAuth", () => ({
  useAuth: () => authState,
}));

vi.mock("../src/lib/authApi", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../src/lib/authApi")>();
  return {
    ...actual,
    startLogin: vi.fn(),
    startRegistration: vi.fn(),
  };
});

describe("AuthPage", () => {
  beforeEach(() => {
    authState.status = "unauthenticated";
    authState.user = null;
    vi.mocked(startRegistration).mockReset();
  });

  it("starts signup with a normalized frontend return target", async () => {
    window.history.pushState({}, "", "/signup?return_to=/dashboard");
    vi.mocked(startRegistration).mockResolvedValue({
      status: "registration_started",
      auth_url: "/api/auth/google?return_to=%2Fdashboard",
      email_sent: true,
    });

    render(<AuthPage mode="signup" />);

    await userEvent.type(
      screen.getByLabelText("Email address"),
      "friend@example.com",
    );
    await userEvent.click(
      screen.getByRole("button", { name: "Send verification link" }),
    );

    expect(startRegistration).toHaveBeenCalledWith(
      "friend@example.com",
      "/dashboard",
    );
    expect(
      await screen.findByText("Check your inbox for a verification link."),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Continue with secure auth" }),
    ).toHaveAttribute("href", "/api/auth/google?return_to=%2Fdashboard");
  });

  it("renders login with a Google auth link for the requested page", () => {
    window.history.pushState({}, "", "/login?return_to=/events/event-123");

    render(<AuthPage mode="login" />);

    expect(screen.getByRole("heading", { name: "Welcome back" })).toBeVisible();
    expect(screen.getByRole("link", { name: "Sign in with Google" })).toHaveAttribute(
      "href",
      "/api/auth/google?return_to=%2Fevents%2Fevent-123",
    );
  });
});
