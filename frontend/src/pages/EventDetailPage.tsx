import { useEffect, useMemo, useState } from "react";

import { EventDetail, fetchEventDetail } from "../lib/eventsApi";

type LoadState = "loading" | "loaded" | "error";

type EventDetailPageProps = {
  eventId: string;
};

function EventDetailPage({ eventId }: EventDetailPageProps) {
  const [detail, setDetail] = useState<EventDetail | null>(null);
  const [status, setStatus] = useState<LoadState>("loading");
  const [message, setMessage] = useState("");

  useEffect(() => {
    let active = true;

    async function loadEvent() {
      setStatus("loading");
      setMessage("");

      try {
        const result = await fetchEventDetail(eventId);
        if (!active) {
          return;
        }
        setDetail(result.event);
        setStatus("loaded");
      } catch (error) {
        if (!active) {
          return;
        }
        setDetail(null);
        setStatus("error");
        setMessage(
          error instanceof Error ? error.message : "Event could not be loaded.",
        );
      }
    }

    void loadEvent();

    return () => {
      active = false;
    };
  }, [eventId]);

  if (status === "loading") {
    return (
      <main className="grid min-h-screen place-items-center bg-slate-50 px-6 text-slate-950">
        <section className="rounded-lg border border-slate-200 bg-white p-6 text-center shadow-sm">
          <p className="text-sm font-semibold text-slate-900">Loading event</p>
          <p className="mt-2 text-sm text-slate-600">
            Event details and attachments are being prepared.
          </p>
        </section>
      </main>
    );
  }

  if (status === "error" || !detail) {
    return (
      <main className="grid min-h-screen place-items-center bg-slate-50 px-6 text-slate-950">
        <section className="w-full max-w-md rounded-lg border border-rose-200 bg-white p-6 text-center shadow-sm">
          <p className="text-base font-semibold text-rose-800">Event unavailable</p>
          <p className="mt-2 text-sm leading-6 text-slate-700">{message}</p>
          <a
            className="mt-5 inline-flex h-11 items-center justify-center rounded-md bg-emerald-700 px-4 text-sm font-semibold text-white transition hover:bg-emerald-800"
            href="/dashboard"
          >
            Back to dashboard
          </a>
        </section>
      </main>
    );
  }

  return <LoadedEventDetail detail={detail} />;
}

function LoadedEventDetail({ detail }: { detail: EventDetail }) {
  const event = detail.event;
  const startsAt = useMemo(() => formatDateTime(event.starts_at), [event.starts_at]);
  const endsAt = useMemo(
    () => (event.ends_at ? formatDateTime(event.ends_at) : null),
    [event.ends_at],
  );
  const location = [event.location_name, event.location_address]
    .filter(Boolean)
    .join(" · ");

  return (
    <main className="min-h-screen bg-slate-50 text-slate-950">
      <section className="mx-auto w-full max-w-6xl px-6 py-8 sm:px-10 lg:px-14">
        <header className="flex flex-col gap-4 border-b border-slate-200 pb-6 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <a className="text-base font-semibold text-emerald-700" href="/dashboard">
              Gather
            </a>
            <h1 className="mt-4 max-w-4xl text-3xl font-semibold leading-tight sm:text-5xl">
              {event.title}
            </h1>
            <div className="mt-4 flex flex-wrap gap-2">
              <span className="rounded-full bg-emerald-100 px-3 py-1 text-xs font-semibold uppercase text-emerald-800">
                {detail.viewer_role}
              </span>
              <span className="rounded-full bg-slate-200 px-3 py-1 text-xs font-semibold uppercase text-slate-700">
                {event.timezone || "UTC"}
              </span>
            </div>
          </div>
          <div className="flex flex-col gap-2 sm:flex-row">
            <a
              className="inline-flex h-11 items-center justify-center rounded-md border border-slate-300 bg-white px-4 text-sm font-semibold text-slate-900 transition hover:border-slate-500 hover:bg-slate-100"
              href="/dashboard"
            >
              Dashboard
            </a>
            <a
              className="inline-flex h-11 items-center justify-center rounded-md bg-emerald-700 px-4 text-sm font-semibold text-white transition hover:bg-emerald-800"
              href="/events/new"
            >
              Create event
            </a>
          </div>
        </header>

        <div className="mt-8 grid gap-6 lg:grid-cols-[minmax(0,1fr)_360px]">
          <div className="space-y-6">
            <section className="overflow-hidden rounded-lg border border-slate-200 bg-white shadow-sm">
              {detail.cover_image ? (
                <img
                  alt=""
                  className="aspect-video w-full object-cover"
                  src={detail.cover_image.url}
                />
              ) : (
                <div className="grid aspect-video w-full place-items-center bg-slate-200 text-base font-semibold text-slate-600">
                  No cover image
                </div>
              )}
            </section>

            <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
              <h2 className="text-base font-semibold text-slate-950">Description</h2>
              {event.description ? (
                <p className="mt-3 whitespace-pre-wrap text-base leading-7 text-slate-700">
                  {event.description}
                </p>
              ) : (
                <p className="mt-3 text-sm leading-6 text-slate-600">
                  No description has been added.
                </p>
              )}
            </section>

            <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
              <div className="flex items-center justify-between gap-4">
                <h2 className="text-base font-semibold text-slate-950">Comments</h2>
                <span className="rounded-full bg-slate-100 px-3 py-1 text-xs font-semibold text-slate-600">
                  Later task
                </span>
              </div>
              <div className="mt-4 rounded-md border border-dashed border-slate-300 bg-slate-50 p-4">
                <p className="text-sm font-semibold text-slate-900">
                  Comment thread will appear here.
                </p>
                <p className="mt-1 text-sm leading-6 text-slate-600">
                  This page reserves the event discussion area for the comments feature.
                </p>
              </div>
            </section>
          </div>

          <aside className="space-y-6">
            <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
              <h2 className="text-base font-semibold text-slate-950">When</h2>
              <dl className="mt-4 space-y-4">
                <div>
                  <dt className="text-xs font-semibold uppercase text-slate-500">
                    Starts
                  </dt>
                  <dd className="mt-1 text-sm font-semibold text-slate-950">
                    {startsAt}
                  </dd>
                </div>
                {endsAt ? (
                  <div>
                    <dt className="text-xs font-semibold uppercase text-slate-500">
                      Ends
                    </dt>
                    <dd className="mt-1 text-sm font-semibold text-slate-950">
                      {endsAt}
                    </dd>
                  </div>
                ) : null}
              </dl>
            </section>

            <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
              <h2 className="text-base font-semibold text-slate-950">Where</h2>
              <p className="mt-3 text-sm leading-6 text-slate-700">
                {location || "No location has been added."}
              </p>
            </section>

            <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
              <h2 className="text-base font-semibold text-slate-950">Attachments</h2>
              {detail.pdf_attachments.length ? (
                <div className="mt-4 space-y-3">
                  {detail.pdf_attachments.map((attachment, index) => (
                    <a
                      className="block rounded-md border border-slate-200 bg-slate-50 p-3 transition hover:border-slate-400 hover:bg-white"
                      href={attachment.url}
                      key={attachment.object_key}
                      rel="noreferrer"
                      target="_blank"
                    >
                      <span className="text-sm font-semibold text-slate-950">
                        PDF attachment {index + 1}
                      </span>
                      <span className="mt-1 block break-all text-xs text-slate-500">
                        {fileNameFromKey(attachment.object_key)}
                      </span>
                    </a>
                  ))}
                </div>
              ) : (
                <p className="mt-3 text-sm leading-6 text-slate-600">
                  No PDF attachments have been added.
                </p>
              )}
            </section>

            <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
              <div className="flex items-center justify-between gap-4">
                <h2 className="text-base font-semibold text-slate-950">RSVPs</h2>
                <span className="rounded-full bg-slate-100 px-3 py-1 text-xs font-semibold text-slate-600">
                  Later task
                </span>
              </div>
              <div className="mt-4 rounded-md border border-dashed border-slate-300 bg-slate-50 p-4">
                <p className="text-sm font-semibold text-slate-900">
                  RSVP list will appear here.
                </p>
                <p className="mt-1 text-sm leading-6 text-slate-600">
                  Guest response tracking will connect to this section.
                </p>
              </div>
            </section>
          </aside>
        </div>
      </section>
    </main>
  );
}

function formatDateTime(value: string): string {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

function fileNameFromKey(key: string): string {
  const segments = key.split("/").filter(Boolean);
  return segments[segments.length - 1] || key;
}

export default EventDetailPage;
