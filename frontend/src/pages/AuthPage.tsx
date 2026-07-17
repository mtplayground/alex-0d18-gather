import { FormEvent, useEffect, useMemo, useState } from "react";

import { useAuth } from "../auth/useAuth";
import {
  AuthStartResponse,
  googleAuthUrl,
  normalizeReturnTo,
  startLogin,
  startRegistration,
} from "../lib/authApi";

type AuthMode = "login" | "signup";
type FormStatus = "idle" | "submitting" | "success" | "error";

type AuthPageProps = {
  mode: AuthMode;
};

const copy = {
  login: {
    title: "Welcome back",
    body: "Continue to your events, invitations, and RSVP updates.",
    emailLabel: "Email address",
    submit: "Continue with email",
    google: "Sign in with Google",
    alternate: "Need an account?",
    alternateHref: "/signup",
    alternateAction: "Create one",
  },
  signup: {
    title: "Create your Gather account",
    body: "Start planning events and inviting friends with one shared place for every detail.",
    emailLabel: "Email address",
    submit: "Send verification link",
    google: "Sign up with Google",
    alternate: "Already have an account?",
    alternateHref: "/login",
    alternateAction: "Sign in",
  },
} satisfies Record<AuthMode, Record<string, string>>;

function AuthPage({ mode }: AuthPageProps) {
  const { status: authStatus } = useAuth();
  const [email, setEmail] = useState("");
  const [status, setStatus] = useState<FormStatus>("idle");
  const [message, setMessage] = useState("");
  const [authUrl, setAuthUrl] = useState("");
  const content = copy[mode];
  const returnTo = useMemo(() => {
    const params = new URLSearchParams(window.location.search);
    return normalizeReturnTo(params.get("return_to"));
  }, []);
  const googleUrl = useMemo(() => googleAuthUrl(returnTo), [returnTo]);

  useEffect(() => {
    if (authStatus === "authenticated") {
      window.location.replace(returnTo);
    }
  }, [authStatus, returnTo]);

  async function submitEmail(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setStatus("submitting");
    setMessage("");
    setAuthUrl("");

    try {
      const result =
        mode === "signup"
          ? await startRegistration(email, returnTo)
          : await startLogin(email, returnTo);
      handleAuthResult(result);
    } catch (error) {
      setStatus("error");
      setMessage(error instanceof Error ? error.message : "Auth request failed.");
    }
  }

  function handleAuthResult(result: AuthStartResponse) {
    if (mode === "login") {
      window.location.assign(result.auth_url);
      return;
    }

    setStatus("success");
    setAuthUrl(result.auth_url);
    setMessage(
      result.email_sent
        ? "Check your inbox for a verification link."
        : "Email delivery is not configured here. Continue with secure auth instead.",
    );
  }

  return (
    <main className="min-h-screen bg-slate-50 text-slate-950">
      <div className="mx-auto grid min-h-screen w-full max-w-6xl grid-cols-1 lg:grid-cols-[1fr_420px]">
        <section className="flex flex-col justify-center px-6 py-10 sm:px-10 lg:px-14">
          <a className="text-base font-semibold text-emerald-700" href="/">
            Gather
          </a>
          <div className="mt-12 max-w-md">
            <h1 className="text-3xl font-semibold leading-tight sm:text-5xl">
              {content.title}
            </h1>
            <p className="mt-4 text-base leading-7 text-slate-700">{content.body}</p>
          </div>

          <form className="mt-8 max-w-md space-y-5" onSubmit={submitEmail}>
            <label className="block">
              <span className="text-sm font-medium text-slate-800">
                {content.emailLabel}
              </span>
              <input
                autoComplete="email"
                className="mt-2 h-12 w-full rounded-md border border-slate-300 bg-white px-4 text-base outline-none transition focus:border-emerald-600 focus:ring-4 focus:ring-emerald-100"
                inputMode="email"
                onChange={(event) => setEmail(event.target.value)}
                placeholder="you@example.com"
                required
                type="email"
                value={email}
              />
            </label>

            <button
              className="h-12 w-full rounded-md bg-emerald-700 px-4 text-base font-semibold text-white transition hover:bg-emerald-800 disabled:cursor-not-allowed disabled:bg-slate-400"
              disabled={status === "submitting"}
              type="submit"
            >
              {status === "submitting" ? "Working..." : content.submit}
            </button>

            <div className="flex items-center gap-3 text-sm text-slate-500">
              <span className="h-px flex-1 bg-slate-200" />
              <span>or</span>
              <span className="h-px flex-1 bg-slate-200" />
            </div>

            <a
              className="flex h-12 w-full items-center justify-center gap-3 rounded-md border border-slate-300 bg-white px-4 text-base font-semibold text-slate-900 transition hover:border-slate-500 hover:bg-slate-100"
              href={googleUrl}
            >
              <span
                aria-hidden="true"
                className="grid h-6 w-6 place-items-center rounded-full bg-slate-950 text-xs font-semibold text-white"
              >
                G
              </span>
              {content.google}
            </a>

            {message ? (
              <div
                className={`rounded-md border px-4 py-3 text-sm leading-6 ${
                  status === "error"
                    ? "border-rose-200 bg-rose-50 text-rose-800"
                    : "border-emerald-200 bg-emerald-50 text-emerald-900"
                }`}
                role="alert"
              >
                <p>{message}</p>
                {authUrl ? (
                  <a className="mt-2 inline-block font-semibold" href={authUrl}>
                    Continue with secure auth
                  </a>
                ) : null}
              </div>
            ) : null}
          </form>

          <p className="mt-8 text-sm text-slate-600">
            {content.alternate}{" "}
            <a className="font-semibold text-emerald-700" href={content.alternateHref}>
              {content.alternateAction}
            </a>
          </p>
        </section>

        <AuthPreview />
      </div>
    </main>
  );
}

function AuthPreview() {
  return (
    <aside className="hidden border-l border-slate-200 bg-white px-8 py-10 lg:flex lg:flex-col lg:justify-center">
      <div className="rounded-lg border border-slate-200 bg-slate-50 p-5 shadow-sm">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm font-semibold text-slate-900">Dinner party</p>
            <p className="mt-1 text-sm text-slate-600">Saturday, 7:30 PM</p>
          </div>
          <span className="rounded-full bg-emerald-100 px-3 py-1 text-xs font-semibold text-emerald-800">
            Live
          </span>
        </div>
        <div className="mt-6 grid grid-cols-3 gap-3">
          {[
            ["12", "Invited"],
            ["8", "Going"],
            ["3", "Notes"],
          ].map(([value, label]) => (
            <div className="rounded-md bg-white p-3" key={label}>
              <p className="text-2xl font-semibold text-slate-950">{value}</p>
              <p className="mt-1 text-xs text-slate-500">{label}</p>
            </div>
          ))}
        </div>
        <div className="mt-6 space-y-3">
          {["Maya confirmed", "Leo asked about parking", "Nora opened invite"].map(
            (item) => (
              <div
                className="flex items-center gap-3 rounded-md bg-white p-3"
                key={item}
              >
                <span className="h-2.5 w-2.5 rounded-full bg-emerald-600" />
                <p className="text-sm text-slate-700">{item}</p>
              </div>
            ),
          )}
        </div>
      </div>
    </aside>
  );
}

export default AuthPage;
