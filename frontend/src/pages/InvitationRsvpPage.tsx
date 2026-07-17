import { useMemo, useState } from "react";

import {
  EventRecord,
  RsvpStatus,
  RsvpUpdateResult,
  updateInvitationRsvp,
} from "../lib/eventsApi";

type InvitationRsvpPageProps = {
  shareToken: string;
};

type SubmitState = "idle" | "saving" | "success" | "error";

const rsvpOptions: Array<{ status: RsvpStatus; label: string; tone: string }> = [
  {
    status: "yes",
    label: "Yes",
    tone: "border-emerald-200 bg-emerald-700 text-white hover:bg-emerald-800",
  },
  {
    status: "maybe",
    label: "Maybe",
    tone: "border-amber-200 bg-amber-500 text-white hover:bg-amber-600",
  },
  {
    status: "no",
    label: "No",
    tone: "border-rose-200 bg-rose-600 text-white hover:bg-rose-700",
  },
];

function InvitationRsvpPage({ shareToken }: InvitationRsvpPageProps) {
  const [state, setState] = useState<SubmitState>("idle");
  const [message, setMessage] = useState("");
  const [selectedStatus, setSelectedStatus] = useState<RsvpStatus | null>(null);
  const [result, setResult] = useState<RsvpUpdateResult | null>(null);
  const event = result?.event ?? null;
  const eventTime = useMemo(() => (event ? formatEventTime(event) : null), [event]);
  const location = event
    ? [event.location_name, event.location_address].filter(Boolean).join(" · ")
    : "";

  async function submitRsvp(status: RsvpStatus) {
    setState("saving");
    setMessage("");
    setSelectedStatus(status);

    try {
      const response = await updateInvitationRsvp(shareToken, status);
      setResult(response);
      setState("success");
      setMessage(
        response.email_sent
          ? "Your RSVP is saved and a confirmation email is on its way."
          : "Your RSVP is saved.",
      );
    } catch (error) {
      setState("error");
      setMessage(error instanceof Error ? error.message : "RSVP could not be saved.");
    }
  }

  return (
    <main className="min-h-screen bg-slate-50 text-slate-950">
      <section className="mx-auto grid min-h-screen w-full max-w-3xl content-center px-6 py-10 sm:px-10">
        <div className="rounded-lg border border-slate-200 bg-white p-6 shadow-sm sm:p-8">
          <a className="text-base font-semibold text-emerald-700" href="/dashboard">
            Gather
          </a>
          <h1 className="mt-5 text-3xl font-semibold leading-tight sm:text-5xl">
            RSVP to this invitation
          </h1>

          {event ? (
            <section className="mt-6 rounded-lg border border-slate-200 bg-slate-50 p-5">
              <p className="text-xl font-semibold text-slate-950">{event.title}</p>
              <dl className="mt-4 space-y-3">
                <div>
                  <dt className="text-xs font-semibold uppercase text-slate-500">
                    When
                  </dt>
                  <dd className="mt-1 text-sm font-semibold text-slate-900">
                    {eventTime}
                  </dd>
                </div>
                <div>
                  <dt className="text-xs font-semibold uppercase text-slate-500">
                    Where
                  </dt>
                  <dd className="mt-1 text-sm font-semibold text-slate-900">
                    {location || "Location to be announced"}
                  </dd>
                </div>
              </dl>
            </section>
          ) : (
            <p className="mt-4 text-base leading-7 text-slate-700">
              Choose your response and the event details will appear with your
              confirmation.
            </p>
          )}

          <div className="mt-7 grid gap-3 sm:grid-cols-3">
            {rsvpOptions.map((option) => (
              <button
                className={`h-12 rounded-md border px-4 text-base font-semibold transition disabled:cursor-not-allowed disabled:border-slate-200 disabled:bg-slate-300 disabled:text-slate-600 ${option.tone}`}
                disabled={state === "saving"}
                key={option.status}
                onClick={() => void submitRsvp(option.status)}
                type="button"
              >
                {state === "saving" && selectedStatus === option.status
                  ? "Saving..."
                  : option.label}
              </button>
            ))}
          </div>

          {selectedStatus ? (
            <p className="mt-4 text-sm font-semibold text-slate-700">
              Current response: {rsvpStatusLabel(selectedStatus)}
            </p>
          ) : null}

          {message ? (
            <p
              className={`mt-4 text-sm leading-6 ${
                state === "error" ? "text-rose-700" : "text-emerald-700"
              }`}
              role={state === "error" ? "alert" : "status"}
            >
              {message}
            </p>
          ) : null}

          {event ? (
            <a
              className="mt-6 inline-flex h-11 items-center justify-center rounded-md border border-slate-300 bg-white px-4 text-sm font-semibold text-slate-900 transition hover:border-slate-500 hover:bg-slate-100"
              href="/dashboard"
            >
              Back to dashboard
            </a>
          ) : null}
        </div>
      </section>
    </main>
  );
}

function formatEventTime(event: EventRecord): string {
  const startsAt = formatDateTime(event.starts_at);
  if (!event.ends_at) {
    return startsAt;
  }

  return `${startsAt} - ${formatDateTime(event.ends_at)}`;
}

function formatDateTime(value: string): string {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

function rsvpStatusLabel(status: RsvpStatus): string {
  if (status === "yes") {
    return "Yes";
  }
  if (status === "no") {
    return "No";
  }

  return "Maybe";
}

export default InvitationRsvpPage;
