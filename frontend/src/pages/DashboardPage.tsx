import { useEffect, useMemo, useState } from "react";

import { useAuth } from "../auth/useAuth";
import {
  DashboardEvent,
  DashboardEventsResult,
  fetchDashboardEvents,
} from "../lib/eventsApi";

type LoadState = "loading" | "loaded" | "error";

const emptyDashboard: DashboardEventsResult = {
  upcoming: [],
  past: [],
};

function DashboardPage() {
  const { user } = useAuth();
  const [events, setEvents] = useState<DashboardEventsResult>(emptyDashboard);
  const [status, setStatus] = useState<LoadState>("loading");
  const [message, setMessage] = useState("");
  const name = user?.display_name || user?.full_name || user?.email || "there";
  const totalEvents = events.upcoming.length + events.past.length;

  useEffect(() => {
    let active = true;

    async function loadDashboardEvents() {
      setStatus("loading");
      setMessage("");

      try {
        const result = await fetchDashboardEvents();
        if (!active) {
          return;
        }
        setEvents(result);
        setStatus("loaded");
      } catch (error) {
        if (!active) {
          return;
        }
        setEvents(emptyDashboard);
        setStatus("error");
        setMessage(
          error instanceof Error
            ? error.message
            : "Dashboard events could not be loaded.",
        );
      }
    }

    void loadDashboardEvents();

    return () => {
      active = false;
    };
  }, []);

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
              Your upcoming and past events are organized by timing and role.
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

        <div className="mt-8 grid gap-4 sm:grid-cols-3">
          <Metric label="Upcoming" value={events.upcoming.length} />
          <Metric label="Past" value={events.past.length} />
          <Metric label="Total events" value={totalEvents} />
        </div>

        {status === "error" ? (
          <div
            className="mt-6 rounded-md border border-rose-200 bg-rose-50 px-4 py-3 text-sm leading-6 text-rose-800"
            role="alert"
          >
            {message}
          </div>
        ) : null}

        {status === "loading" ? (
          <section className="mt-8 rounded-lg border border-slate-200 bg-white p-6 text-center shadow-sm">
            <p className="text-sm font-semibold text-slate-900">Loading events</p>
            <p className="mt-2 text-sm text-slate-600">
              Your dashboard will appear once event data is ready.
            </p>
          </section>
        ) : (
          <div className="mt-8 grid gap-8">
            <EventSection
              emptyActionHref="/events/new"
              emptyActionText="Create event"
              emptyText="No upcoming events yet."
              events={events.upcoming}
              title="Upcoming events"
            />
            <EventSection
              emptyText="Past events will appear here after their start time."
              events={events.past}
              title="Past events"
            />
          </div>
        )}
      </section>
    </main>
  );
}

function Metric({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
      <p className="text-3xl font-semibold text-slate-950">{value}</p>
      <p className="mt-1 text-sm font-semibold text-slate-600">{label}</p>
    </div>
  );
}

type EventSectionProps = {
  title: string;
  events: DashboardEvent[];
  emptyText: string;
  emptyActionHref?: string;
  emptyActionText?: string;
};

function EventSection({
  title,
  events,
  emptyText,
  emptyActionHref,
  emptyActionText,
}: EventSectionProps) {
  return (
    <section>
      <div className="flex items-center justify-between gap-4">
        <h2 className="text-xl font-semibold text-slate-950">{title}</h2>
        <span className="text-sm font-semibold text-slate-500">
          {events.length} {events.length === 1 ? "event" : "events"}
        </span>
      </div>

      {events.length ? (
        <div className="mt-4 grid gap-4 md:grid-cols-2">
          {events.map((event) => (
            <EventCard event={event} key={event.id} />
          ))}
        </div>
      ) : (
        <div className="mt-4 rounded-lg border border-dashed border-slate-300 bg-white p-6 text-center shadow-sm">
          <p className="text-sm font-semibold text-slate-900">{emptyText}</p>
          {emptyActionHref && emptyActionText ? (
            <a
              className="mt-4 inline-flex h-10 items-center justify-center rounded-md bg-emerald-700 px-4 text-sm font-semibold text-white transition hover:bg-emerald-800"
              href={emptyActionHref}
            >
              {emptyActionText}
            </a>
          ) : null}
        </div>
      )}
    </section>
  );
}

function EventCard({ event }: { event: DashboardEvent }) {
  const startsAt = useMemo(() => formatDateTime(event.starts_at), [event.starts_at]);
  const endsAt = useMemo(
    () => (event.ends_at ? formatDateTime(event.ends_at) : null),
    [event.ends_at],
  );

  return (
    <article className="overflow-hidden rounded-lg border border-slate-200 bg-white shadow-sm">
      <a className="block transition hover:bg-slate-50" href={`/events/${event.id}`}>
        {event.cover_image ? (
          <img
            alt=""
            className="aspect-video w-full object-cover"
            src={event.cover_image.url}
          />
        ) : (
          <div className="grid aspect-video w-full place-items-center bg-slate-200 text-sm font-semibold text-slate-600">
            No cover image
          </div>
        )}
        <div className="p-5">
          <div className="flex items-start justify-between gap-4">
            <h3 className="text-lg font-semibold leading-7 text-slate-950">
              {event.title}
            </h3>
            <span className="shrink-0 rounded-full bg-emerald-100 px-3 py-1 text-xs font-semibold uppercase text-emerald-800">
              {event.viewer_role}
            </span>
          </div>
          <dl className="mt-4 grid gap-3 text-sm text-slate-600">
            <div>
              <dt className="text-xs font-semibold uppercase text-slate-500">Starts</dt>
              <dd className="mt-1 font-semibold text-slate-900">{startsAt}</dd>
            </div>
            {endsAt ? (
              <div>
                <dt className="text-xs font-semibold uppercase text-slate-500">Ends</dt>
                <dd className="mt-1 font-semibold text-slate-900">{endsAt}</dd>
              </div>
            ) : null}
            <div>
              <dt className="text-xs font-semibold uppercase text-slate-500">
                Location
              </dt>
              <dd className="mt-1 font-semibold text-slate-900">
                {event.location_name || "No location added"}
              </dd>
            </div>
          </dl>
        </div>
      </a>
    </article>
  );
}

function formatDateTime(value: string): string {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

export default DashboardPage;
