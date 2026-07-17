import { googleAuthUrl } from "../lib/authApi";

function HomePage() {
  return (
    <main className="min-h-screen bg-slate-50 text-slate-950">
      <section className="mx-auto grid min-h-screen w-full max-w-6xl grid-cols-1 content-center gap-12 px-6 py-10 sm:px-10 lg:grid-cols-[1.1fr_0.9fr] lg:px-14">
        <div>
          <p className="text-base font-semibold text-emerald-700">Gather</p>
          <h1 className="mt-5 max-w-3xl text-4xl font-semibold leading-tight sm:text-6xl">
            Plan events, invite friends, and keep every RSVP in one place.
          </h1>
          <p className="mt-6 max-w-2xl text-lg leading-8 text-slate-700">
            Create an event, share the invitation, and see responses as they happen
            without chasing separate message threads.
          </p>
          <div className="mt-10 flex flex-col gap-3 sm:flex-row">
            <a
              className="inline-flex h-12 items-center justify-center rounded-md bg-emerald-700 px-5 text-base font-semibold text-white transition hover:bg-emerald-800"
              href="/signup"
            >
              Create account
            </a>
            <a
              className="inline-flex h-12 items-center justify-center rounded-md border border-slate-300 bg-white px-5 text-base font-semibold text-slate-900 transition hover:border-slate-500 hover:bg-slate-100"
              href="/login"
            >
              Sign in
            </a>
            <a
              className="inline-flex h-12 items-center justify-center rounded-md border border-slate-300 bg-white px-5 text-base font-semibold text-slate-900 transition hover:border-slate-500 hover:bg-slate-100"
              href={googleAuthUrl("/")}
            >
              Sign in with Google
            </a>
          </div>
        </div>

        <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
          <div className="flex items-start justify-between gap-4">
            <div>
              <p className="text-sm font-semibold text-slate-900">Weekend brunch</p>
              <p className="mt-1 text-sm text-slate-600">Sun, 11:00 AM</p>
            </div>
            <span className="rounded-full bg-blue-100 px-3 py-1 text-xs font-semibold text-blue-800">
              Draft
            </span>
          </div>
          <div className="mt-6 h-3 rounded-full bg-slate-100">
            <div className="h-3 w-2/3 rounded-full bg-emerald-600" />
          </div>
          <div className="mt-6 grid grid-cols-3 gap-3">
            {[
              ["18", "Guests"],
              ["11", "RSVPs"],
              ["2", "Files"],
            ].map(([value, label]) => (
              <div className="rounded-md bg-slate-50 p-3" key={label}>
                <p className="text-2xl font-semibold">{value}</p>
                <p className="mt-1 text-xs text-slate-500">{label}</p>
              </div>
            ))}
          </div>
        </div>
      </section>
    </main>
  );
}

export default HomePage;
