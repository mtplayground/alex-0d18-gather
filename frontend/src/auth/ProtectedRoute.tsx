import { ReactNode, useEffect } from "react";

import { normalizeReturnTo } from "../lib/authApi";
import { useAuth } from "./useAuth";

type ProtectedRouteProps = {
  children: ReactNode;
};

export function ProtectedRoute({ children }: ProtectedRouteProps) {
  const { status } = useAuth();

  useEffect(() => {
    if (status !== "unauthenticated") {
      return;
    }

    const returnTo = normalizeReturnTo(
      `${window.location.pathname}${window.location.search}`,
    );
    window.location.replace(`/login?return_to=${encodeURIComponent(returnTo)}`);
  }, [status]);

  if (status === "loading" || status === "unauthenticated") {
    return (
      <main className="grid min-h-screen place-items-center bg-slate-50 px-6 text-slate-950">
        <div className="rounded-lg border border-slate-200 bg-white p-6 text-center shadow-sm">
          <p className="text-sm font-semibold text-slate-900">Checking your session</p>
          <p className="mt-2 text-sm text-slate-600">
            You will continue once your session is confirmed.
          </p>
        </div>
      </main>
    );
  }

  return children;
}
