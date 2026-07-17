function App() {
  return (
    <main className="min-h-screen bg-stone-50 text-slate-950">
      <section className="mx-auto flex min-h-screen w-full max-w-5xl flex-col justify-center px-6 py-16">
        <p className="text-sm font-semibold uppercase tracking-wide text-emerald-700">
          Gather
        </p>
        <h1 className="mt-4 max-w-3xl text-4xl font-semibold leading-tight sm:text-6xl">
          Plan events, invite friends, and keep every RSVP in one place.
        </h1>
        <p className="mt-6 max-w-2xl text-lg leading-8 text-slate-700">
          The application shell is ready for the authentication, event, invitation,
          RSVP, comment, and activity workflows planned for the next implementation
          issues.
        </p>
        <div className="mt-10 grid gap-4 sm:grid-cols-3">
          {["Profiles", "Events", "Invitations"].map((label) => (
            <div
              className="rounded-lg border border-stone-200 bg-white p-5 shadow-sm"
              key={label}
            >
              <h2 className="text-base font-semibold">{label}</h2>
              <p className="mt-2 text-sm leading-6 text-slate-600">
                Base route structure is in place for future feature work.
              </p>
            </div>
          ))}
        </div>
      </section>
    </main>
  );
}

export default App;
