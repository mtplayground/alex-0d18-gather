import { useAuth } from "../auth/useAuth";

function DashboardPage() {
  const { user } = useAuth();
  const name = user?.display_name || user?.full_name || user?.email || "there";

  return (
    <main className="min-h-screen bg-slate-50 text-slate-950">
      <section className="mx-auto w-full max-w-6xl px-6 py-8 sm:px-10 lg:px-14">
        <header className="flex flex-col gap-5 border-b border-slate-200 pb-6 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <a className="text-base font-semibold text-emerald-700" href="/">
              Gather
            </a>
            <h1 className="mt-4 text-3xl font-semibold leading-tight sm:text-5xl">
              Welcome back, {name}.
            </h1>
            <p className="mt-3 text-base leading-7 text-slate-700">
              Your protected workspace is ready for events, invitations, and RSVP
              follow-up.
            </p>
          </div>
          <div className="rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
            <p className="text-xs font-semibold uppercase text-slate-500">
              Signed in as
            </p>
            <p className="mt-1 text-sm font-semibold text-slate-950">{user?.email}</p>
            <p className="mt-2 text-xs text-slate-500">
              {user?.email_verified ? "Email verified" : "Email verification pending"}
            </p>
            <div className="mt-4 flex flex-col gap-2">
              <a
                className="inline-flex h-10 items-center justify-center rounded-md bg-emerald-700 px-4 text-sm font-semibold text-white transition hover:bg-emerald-800"
                href="/events/new"
              >
                Create event
              </a>
              <a
                className="inline-flex h-10 items-center justify-center rounded-md border border-slate-300 bg-white px-4 text-sm font-semibold text-slate-900 transition hover:border-slate-500 hover:bg-slate-100"
                href="/profile"
              >
                Profile settings
              </a>
            </div>
          </div>
        </header>

        <div className="mt-8 grid gap-4 md:grid-cols-3">
          {[
            ["Upcoming events", "Create and manage plans from one place."],
            ["Invitations", "Track who has opened, accepted, or declined."],
            ["Activity", "See recent RSVP and comment updates."],
          ].map(([title, body]) => (
            <article
              className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm"
              key={title}
            >
              <h2 className="text-base font-semibold text-slate-950">{title}</h2>
              <p className="mt-2 text-sm leading-6 text-slate-600">{body}</p>
            </article>
          ))}
        </div>
      </section>
    </main>
  );
}

export default DashboardPage;
