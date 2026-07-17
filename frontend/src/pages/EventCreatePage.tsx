import { ChangeEvent, FormEvent, useEffect, useMemo, useState } from "react";

import { useAuth } from "../auth/useAuth";
import { createEvent } from "../lib/eventsApi";

type SaveState = "idle" | "saving" | "success" | "error";

type FormState = {
  title: string;
  description: string;
  startsAt: string;
  endsAt: string;
  timezone: string;
  locationName: string;
  locationAddress: string;
};

const MAX_COVER_IMAGE_BYTES = 8 * 1024 * 1024;
const MAX_PDF_ATTACHMENT_BYTES = 10 * 1024 * 1024;
const MAX_PDF_ATTACHMENTS = 20;
const COVER_IMAGE_TYPES = new Set([
  "image/jpeg",
  "image/png",
  "image/webp",
  "image/gif",
]);

const emptyForm: FormState = {
  title: "",
  description: "",
  startsAt: "",
  endsAt: "",
  timezone: Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC",
  locationName: "",
  locationAddress: "",
};

function EventCreatePage() {
  const { user } = useAuth();
  const [form, setForm] = useState<FormState>(emptyForm);
  const [coverImage, setCoverImage] = useState<File | null>(null);
  const [pdfAttachments, setPdfAttachments] = useState<File[]>([]);
  const [coverPreviewUrl, setCoverPreviewUrl] = useState("");
  const [status, setStatus] = useState<SaveState>("idle");
  const [message, setMessage] = useState("");
  const [createdEventId, setCreatedEventId] = useState("");

  const selectedPdfTotal = useMemo(
    () => pdfAttachments.reduce((total, file) => total + file.size, 0),
    [pdfAttachments],
  );

  useEffect(() => {
    if (!coverImage) {
      setCoverPreviewUrl("");
      return;
    }

    const objectUrl = URL.createObjectURL(coverImage);
    setCoverPreviewUrl(objectUrl);

    return () => {
      URL.revokeObjectURL(objectUrl);
    };
  }, [coverImage]);

  function updateField<K extends keyof FormState>(key: K, value: FormState[K]) {
    setForm((current) => ({ ...current, [key]: value }));
  }

  function chooseCoverImage(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0] ?? null;
    setMessage("");

    if (!file) {
      setCoverImage(null);
      return;
    }

    if (!COVER_IMAGE_TYPES.has(file.type)) {
      setCoverImage(null);
      setStatus("error");
      setMessage("Cover image must be a JPEG, PNG, WEBP, or GIF file.");
      event.target.value = "";
      return;
    }

    if (file.size > MAX_COVER_IMAGE_BYTES) {
      setCoverImage(null);
      setStatus("error");
      setMessage("Cover image must be 8 MB or smaller.");
      event.target.value = "";
      return;
    }

    setStatus("idle");
    setCoverImage(file);
  }

  function choosePdfAttachments(event: ChangeEvent<HTMLInputElement>) {
    const files = Array.from(event.target.files ?? []);
    setMessage("");

    if (files.length > MAX_PDF_ATTACHMENTS) {
      setPdfAttachments([]);
      setStatus("error");
      setMessage(`Attach up to ${MAX_PDF_ATTACHMENTS} PDFs.`);
      event.target.value = "";
      return;
    }

    const invalidFile = files.find((file) => file.type !== "application/pdf");
    if (invalidFile) {
      setPdfAttachments([]);
      setStatus("error");
      setMessage("Attachments must be PDF files.");
      event.target.value = "";
      return;
    }

    const oversizedFile = files.find((file) => file.size > MAX_PDF_ATTACHMENT_BYTES);
    if (oversizedFile) {
      setPdfAttachments([]);
      setStatus("error");
      setMessage("Each PDF must be 10 MB or smaller.");
      event.target.value = "";
      return;
    }

    setStatus("idle");
    setPdfAttachments(files);
  }

  async function submitEvent(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setStatus("saving");
    setMessage("");
    setCreatedEventId("");

    try {
      const result = await createEvent({
        ...form,
        coverImage,
        pdfAttachments,
      });
      setStatus("success");
      setCreatedEventId(result.event.id);
      setMessage("Event created.");
    } catch (error) {
      setStatus("error");
      setMessage(
        error instanceof Error ? error.message : "Event could not be created.",
      );
    }
  }

  return (
    <main className="min-h-screen bg-slate-50 text-slate-950">
      <section className="mx-auto w-full max-w-6xl px-6 py-8 sm:px-10 lg:px-14">
        <header className="flex flex-col gap-4 border-b border-slate-200 pb-6 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <a className="text-base font-semibold text-emerald-700" href="/dashboard">
              Gather
            </a>
            <h1 className="mt-4 text-3xl font-semibold leading-tight sm:text-5xl">
              Create event
            </h1>
            <p className="mt-3 max-w-2xl text-base leading-7 text-slate-700">
              Publish the core event details, cover image, and host attachments from one
              place.
            </p>
          </div>
          <a
            className="inline-flex h-11 items-center justify-center rounded-md border border-slate-300 bg-white px-4 text-sm font-semibold text-slate-900 transition hover:border-slate-500 hover:bg-slate-100"
            href="/dashboard"
          >
            Back to dashboard
          </a>
        </header>

        <form
          className="mt-8 grid gap-6 lg:grid-cols-[minmax(0,1fr)_360px]"
          onSubmit={submitEvent}
        >
          <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
            <div className="grid gap-5">
              <TextInput
                label="Event title"
                maxLength={200}
                onChange={(value) => updateField("title", value)}
                required
                value={form.title}
              />

              <label className="block">
                <span className="text-sm font-medium text-slate-800">Description</span>
                <textarea
                  className="mt-2 min-h-40 w-full rounded-md border border-slate-300 bg-white px-4 py-3 text-base outline-none transition focus:border-emerald-600 focus:ring-4 focus:ring-emerald-100"
                  maxLength={5000}
                  onChange={(event) => updateField("description", event.target.value)}
                  value={form.description}
                />
              </label>

              <div className="grid gap-5 md:grid-cols-2">
                <TextInput
                  label="Starts"
                  onChange={(value) => updateField("startsAt", value)}
                  required
                  type="datetime-local"
                  value={form.startsAt}
                />
                <TextInput
                  label="Ends"
                  onChange={(value) => updateField("endsAt", value)}
                  type="datetime-local"
                  value={form.endsAt}
                />
              </div>

              <TextInput
                label="Timezone"
                maxLength={100}
                onChange={(value) => updateField("timezone", value)}
                value={form.timezone}
              />

              <div className="grid gap-5 md:grid-cols-2">
                <TextInput
                  label="Location name"
                  maxLength={200}
                  onChange={(value) => updateField("locationName", value)}
                  value={form.locationName}
                />
                <TextInput
                  label="Location address"
                  maxLength={500}
                  onChange={(value) => updateField("locationAddress", value)}
                  value={form.locationAddress}
                />
              </div>
            </div>
          </section>

          <aside className="space-y-6">
            <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
              <h2 className="text-base font-semibold text-slate-950">Cover image</h2>
              <div className="mt-4 overflow-hidden rounded-md border border-slate-200 bg-slate-100">
                {coverPreviewUrl ? (
                  <img
                    alt=""
                    className="aspect-video w-full object-cover"
                    src={coverPreviewUrl}
                  />
                ) : (
                  <div className="grid aspect-video w-full place-items-center text-sm font-semibold text-slate-500">
                    No cover selected
                  </div>
                )}
              </div>
              <input
                accept="image/jpeg,image/png,image/webp,image/gif"
                className="mt-4 block w-full text-sm text-slate-700 file:mr-4 file:rounded-md file:border-0 file:bg-slate-900 file:px-4 file:py-2 file:text-sm file:font-semibold file:text-white"
                onChange={chooseCoverImage}
                type="file"
              />
              {coverImage ? (
                <p className="mt-3 text-sm text-slate-600">
                  {coverImage.name} · {formatFileSize(coverImage.size)}
                </p>
              ) : null}
            </section>

            <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
              <h2 className="text-base font-semibold text-slate-950">
                PDF attachments
              </h2>
              <input
                accept="application/pdf"
                className="mt-4 block w-full text-sm text-slate-700 file:mr-4 file:rounded-md file:border-0 file:bg-slate-900 file:px-4 file:py-2 file:text-sm file:font-semibold file:text-white"
                multiple
                onChange={choosePdfAttachments}
                type="file"
              />
              {pdfAttachments.length ? (
                <div className="mt-4 space-y-3">
                  {pdfAttachments.map((file) => (
                    <div
                      className="rounded-md border border-slate-200 bg-slate-50 p-3"
                      key={`${file.name}-${file.size}-${file.lastModified}`}
                    >
                      <p className="text-sm font-semibold text-slate-900">
                        {file.name}
                      </p>
                      <p className="mt-1 text-xs text-slate-500">
                        {formatFileSize(file.size)}
                      </p>
                    </div>
                  ))}
                  <p className="text-sm text-slate-600">
                    Total {formatFileSize(selectedPdfTotal)}
                  </p>
                </div>
              ) : null}
            </section>

            <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
              <p className="text-sm text-slate-600">
                Signed in as{" "}
                <span className="font-semibold text-slate-950">{user?.email}</span>
              </p>
              <button
                className="mt-4 h-12 w-full rounded-md bg-emerald-700 px-4 text-base font-semibold text-white transition hover:bg-emerald-800 disabled:cursor-not-allowed disabled:bg-slate-400"
                disabled={status === "saving"}
                type="submit"
              >
                {status === "saving" ? "Creating..." : "Create event"}
              </button>
              {message ? (
                <div
                  className={`mt-4 rounded-md border px-4 py-3 text-sm leading-6 ${
                    status === "error"
                      ? "border-rose-200 bg-rose-50 text-rose-800"
                      : "border-emerald-200 bg-emerald-50 text-emerald-900"
                  }`}
                  role="alert"
                >
                  <p>{message}</p>
                  {createdEventId ? (
                    <a
                      className="mt-2 inline-block font-semibold"
                      href={`/events/${createdEventId}`}
                    >
                      Open event detail
                    </a>
                  ) : null}
                </div>
              ) : null}
            </section>
          </aside>
        </form>
      </section>
    </main>
  );
}

type TextInputProps = {
  label: string;
  value: string;
  onChange: (value: string) => void;
  type?: string;
  required?: boolean;
  maxLength?: number;
};

function TextInput({
  label,
  value,
  onChange,
  type = "text",
  required = false,
  maxLength,
}: TextInputProps) {
  return (
    <label className="block">
      <span className="text-sm font-medium text-slate-800">{label}</span>
      <input
        className="mt-2 h-12 w-full rounded-md border border-slate-300 bg-white px-4 text-base outline-none transition focus:border-emerald-600 focus:ring-4 focus:ring-emerald-100"
        maxLength={maxLength}
        onChange={(event) => onChange(event.target.value)}
        required={required}
        type={type}
        value={value}
      />
    </label>
  );
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024 * 1024) {
    return `${Math.max(1, Math.round(bytes / 1024))} KB`;
  }

  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export default EventCreatePage;
