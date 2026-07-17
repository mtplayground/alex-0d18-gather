export type EventRecord = {
  id: string;
  organizer_user_id: string;
  title: string;
  description: string | null;
  starts_at: string;
  ends_at: string | null;
  timezone: string | null;
  location_name: string | null;
  location_address: string | null;
  cover_image_object_key: string | null;
  pdf_attachment_object_keys: string[];
  created_at: string;
  updated_at: string;
};

export type CreateEventResult = {
  status: string;
  event: EventRecord;
};

export type EventCreateInput = {
  title: string;
  description: string;
  startsAt: string;
  endsAt: string;
  timezone: string;
  locationName: string;
  locationAddress: string;
  coverImage: File | null;
  pdfAttachments: File[];
};

export async function createEvent(input: EventCreateInput): Promise<CreateEventResult> {
  const form = new FormData();
  form.append("title", input.title);
  form.append("starts_at", toRfc3339(input.startsAt, "Start date and time"));

  appendIfPresent(form, "description", input.description);
  appendIfPresent(
    form,
    "ends_at",
    input.endsAt ? toRfc3339(input.endsAt, "End date and time") : "",
  );
  appendIfPresent(form, "timezone", input.timezone);
  appendIfPresent(form, "location_name", input.locationName);
  appendIfPresent(form, "location_address", input.locationAddress);

  if (input.coverImage) {
    form.append("cover_image", input.coverImage);
  }

  for (const file of input.pdfAttachments) {
    form.append("pdf_attachments", file);
  }

  const response = await fetch("/api/events/", {
    method: "POST",
    credentials: "include",
    body: form,
  });
  const payload = (await response.json().catch(() => null)) as
    (CreateEventResult & { message?: string }) | { message?: string } | null;

  if (!response.ok) {
    throw new Error(readErrorMessage(payload, "Event could not be created."));
  }

  if (!payload || !("event" in payload)) {
    throw new Error("The event creation response was not recognized.");
  }

  return payload;
}

function appendIfPresent(form: FormData, key: string, value: string) {
  const trimmed = value.trim();
  if (trimmed) {
    form.append(key, trimmed);
  }
}

function toRfc3339(value: string, label: string): string {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    throw new Error(`${label} is invalid.`);
  }

  return date.toISOString();
}

function readErrorMessage(
  payload: { message?: string } | null,
  fallback: string,
): string {
  return payload?.message || fallback;
}
