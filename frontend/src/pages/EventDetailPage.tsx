import { FormEvent, useEffect, useMemo, useState } from "react";

import {
  createShareableInviteLink,
  EventRsvpListResult,
  EventDetail,
  fetchEventRsvps,
  fetchEventDetail,
  inviteFriendByEmail,
  RsvpListEntry,
} from "../lib/eventsApi";

type LoadState = "loading" | "loaded" | "error";
type InviteActionState = "idle" | "sending" | "linking" | "success" | "error";

type EventDetailPageProps = {
  eventId: string;
};

const emptyRsvpList: EventRsvpListResult = {
  event_id: "",
  coming: [],
  declined: [],
  maybe: [],
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

            <InviteFriendsPanel eventId={event.id} />

            <RsvpListPanel eventId={event.id} />
          </aside>
        </div>
      </section>
    </main>
  );
}

function RsvpListPanel({ eventId }: { eventId: string }) {
  const [rsvps, setRsvps] = useState<EventRsvpListResult>(emptyRsvpList);
  const [status, setStatus] = useState<LoadState>("loading");
  const [message, setMessage] = useState("");
  const totalResponses =
    rsvps.coming.length + rsvps.maybe.length + rsvps.declined.length;

  useEffect(() => {
    let active = true;

    async function loadRsvps() {
      setStatus("loading");
      setMessage("");

      try {
        const result = await fetchEventRsvps(eventId);
        if (!active) {
          return;
        }
        setRsvps(result);
        setStatus("loaded");
      } catch (error) {
        if (!active) {
          return;
        }
        setRsvps({ ...emptyRsvpList, event_id: eventId });
        setStatus("error");
        setMessage(
          error instanceof Error ? error.message : "RSVP list could not be loaded.",
        );
      }
    }

    void loadRsvps();

    return () => {
      active = false;
    };
  }, [eventId]);

  return (
    <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
      <div className="flex items-center justify-between gap-4">
        <h2 className="text-base font-semibold text-slate-950">RSVPs</h2>
        <span className="rounded-full bg-slate-100 px-3 py-1 text-xs font-semibold text-slate-600">
          {totalResponses} {totalResponses === 1 ? "reply" : "replies"}
        </span>
      </div>

      {status === "loading" ? (
        <p className="mt-4 text-sm leading-6 text-slate-600">Loading RSVP list.</p>
      ) : null}

      {status === "error" ? (
        <p className="mt-4 text-sm leading-6 text-rose-700" role="alert">
          {message}
        </p>
      ) : null}

      {status === "loaded" && totalResponses === 0 ? (
        <div className="mt-4 rounded-md border border-dashed border-slate-300 bg-slate-50 p-4">
          <p className="text-sm font-semibold text-slate-900">No RSVP replies yet.</p>
          <p className="mt-1 text-sm leading-6 text-slate-600">
            Guest responses will appear here after they choose yes, maybe, or no.
          </p>
        </div>
      ) : null}

      {status === "loaded" && totalResponses > 0 ? (
        <div className="mt-4 space-y-5">
          <RsvpGroup entries={rsvps.coming} tone="emerald" title="Coming" />
          <RsvpGroup entries={rsvps.maybe} tone="amber" title="Maybe" />
          <RsvpGroup entries={rsvps.declined} tone="rose" title="Declined" />
        </div>
      ) : null}
    </section>
  );
}

function RsvpGroup({
  entries,
  title,
  tone,
}: {
  entries: RsvpListEntry[];
  title: string;
  tone: "emerald" | "amber" | "rose";
}) {
  const toneClass =
    tone === "emerald"
      ? "bg-emerald-100 text-emerald-800"
      : tone === "amber"
        ? "bg-amber-100 text-amber-800"
        : "bg-rose-100 text-rose-800";

  return (
    <section>
      <div className="flex items-center justify-between gap-3">
        <h3 className="text-sm font-semibold text-slate-950">{title}</h3>
        <span className={`rounded-full px-2.5 py-1 text-xs font-semibold ${toneClass}`}>
          {entries.length}
        </span>
      </div>
      {entries.length ? (
        <div className="mt-3 space-y-2">
          {entries.map((entry) => (
            <div
              className="rounded-md border border-slate-200 bg-slate-50 p-3"
              key={entry.invitation_id}
            >
              <p className="truncate text-sm font-semibold text-slate-950">
                {rsvpEntryName(entry)}
              </p>
              <p className="mt-1 text-xs text-slate-500">
                {formatDateTime(entry.rsvp_responded_at)}
              </p>
            </div>
          ))}
        </div>
      ) : (
        <p className="mt-2 text-sm leading-6 text-slate-600">No replies.</p>
      )}
    </section>
  );
}

function InviteFriendsPanel({ eventId }: { eventId: string }) {
  const [email, setEmail] = useState("");
  const [status, setStatus] = useState<InviteActionState>("idle");
  const [message, setMessage] = useState("");
  const [shareLink, setShareLink] = useState("");

  const isWorking = status === "sending" || status === "linking";

  async function sendInvite(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await runInviteAction("sending");
  }

  async function createLink() {
    await runInviteAction("linking");
  }

  async function runInviteAction(action: "sending" | "linking") {
    const inviteeEmail = email.trim();
    if (!inviteeEmail) {
      setStatus("error");
      setMessage("Enter an email address first.");
      return;
    }

    setStatus(action);
    setMessage("");

    try {
      const result =
        action === "sending"
          ? await inviteFriendByEmail(eventId, inviteeEmail)
          : await createShareableInviteLink(eventId, inviteeEmail);
      setShareLink(result.invitation_url);
      setStatus("success");
      setMessage(
        action === "sending" && result.email_sent
          ? `Invitation sent to ${inviteeEmail}.`
          : `Invitation link created for ${inviteeEmail}.`,
      );
    } catch (error) {
      setStatus("error");
      setMessage(
        error instanceof Error ? error.message : "Invitation could not be prepared.",
      );
    }
  }

  async function copyShareLink() {
    if (!shareLink) {
      return;
    }

    try {
      await navigator.clipboard.writeText(shareLink);
      setStatus("success");
      setMessage("Shareable link copied.");
    } catch {
      setStatus("error");
      setMessage("Copy failed. Select the link field and copy it manually.");
    }
  }

  return (
    <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
      <h2 className="text-base font-semibold text-slate-950">Invite friends</h2>
      <form className="mt-4 space-y-4" onSubmit={sendInvite}>
        <label className="block">
          <span className="text-sm font-medium text-slate-800">Email address</span>
          <input
            className="mt-2 h-11 w-full rounded-md border border-slate-300 bg-white px-3 text-sm outline-none transition focus:border-emerald-600 focus:ring-4 focus:ring-emerald-100"
            disabled={isWorking}
            inputMode="email"
            maxLength={320}
            onChange={(event) => setEmail(event.target.value)}
            placeholder="friend@example.com"
            required
            type="email"
            value={email}
          />
        </label>

        <div className="grid gap-2 sm:grid-cols-2">
          <button
            className="inline-flex h-11 items-center justify-center rounded-md bg-emerald-700 px-4 text-sm font-semibold text-white transition hover:bg-emerald-800 disabled:cursor-not-allowed disabled:bg-slate-400"
            disabled={isWorking}
            type="submit"
          >
            {status === "sending" ? "Sending..." : "Send invite"}
          </button>
          <button
            className="inline-flex h-11 items-center justify-center rounded-md border border-slate-300 bg-white px-4 text-sm font-semibold text-slate-900 transition hover:border-slate-500 hover:bg-slate-100 disabled:cursor-not-allowed disabled:bg-slate-100 disabled:text-slate-500"
            disabled={isWorking}
            onClick={createLink}
            type="button"
          >
            {status === "linking" ? "Creating..." : "Create link"}
          </button>
        </div>
      </form>

      {shareLink ? (
        <div className="mt-4 space-y-2">
          <label className="block">
            <span className="text-sm font-medium text-slate-800">Shareable link</span>
            <input
              className="mt-2 h-11 w-full rounded-md border border-slate-300 bg-slate-50 px-3 text-sm text-slate-700"
              readOnly
              type="text"
              value={shareLink}
            />
          </label>
          <button
            className="inline-flex h-10 w-full items-center justify-center rounded-md border border-slate-300 bg-white px-4 text-sm font-semibold text-slate-900 transition hover:border-slate-500 hover:bg-slate-100"
            onClick={copyShareLink}
            type="button"
          >
            Copy link
          </button>
        </div>
      ) : null}

      {message ? (
        <p
          className={`mt-4 text-sm leading-6 ${
            status === "error" ? "text-rose-700" : "text-emerald-700"
          }`}
          role={status === "error" ? "alert" : "status"}
        >
          {message}
        </p>
      ) : null}
    </section>
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

function rsvpEntryName(entry: RsvpListEntry): string {
  return entry.invitee_email || entry.invitee_user_id || "Guest";
}

export default EventDetailPage;
