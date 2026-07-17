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

export type EventAsset = {
  object_key: string;
  url: string;
};

export type EventViewerRole = "host" | "guest";

export type EventDetail = {
  event: EventRecord;
  viewer_role: EventViewerRole;
  cover_image: EventAsset | null;
  pdf_attachments: EventAsset[];
};

export type EventDetailResult = {
  event: EventDetail;
};

export type DashboardEvent = {
  id: string;
  title: string;
  starts_at: string;
  ends_at: string | null;
  timezone: string | null;
  location_name: string | null;
  cover_image: EventAsset | null;
  viewer_role: EventViewerRole;
};

export type DashboardEventsResult = {
  upcoming: DashboardEvent[];
  past: DashboardEvent[];
};

export type InvitationRecord = {
  id: string;
  event_id: string;
  invitee_user_id: string | null;
  invitee_email: string | null;
  status: string;
  share_token: string;
  rsvp_status: string | null;
  rsvp_responded_at: string | null;
  created_at: string;
  updated_at: string;
};

export type InviteFriendResult = {
  status: string;
  invitation: InvitationRecord;
  invitation_url: string;
  email_sent?: boolean;
};

export type RsvpStatus = "yes" | "no" | "maybe";

export type RsvpListEntry = {
  invitation_id: string;
  invitee_user_id: string | null;
  invitee_email: string | null;
  rsvp_status: RsvpStatus;
  rsvp_responded_at: string;
};

export type EventRsvpListResult = {
  event_id: string;
  coming: RsvpListEntry[];
  declined: RsvpListEntry[];
  maybe: RsvpListEntry[];
};

export type EventComment = {
  id: string;
  event_id: string;
  author_user_id: string;
  body: string;
  created_at: string;
  updated_at: string;
};

export type EventCommentsResult = {
  comments: EventComment[];
};

export type CreateCommentResult = {
  status: string;
  comment: EventComment;
};

export type RsvpUpdateResult = {
  status: string;
  invitation: InvitationRecord;
  event: EventRecord;
  email_sent: boolean;
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

export async function fetchEventDetail(eventId: string): Promise<EventDetailResult> {
  const response = await fetch(`/api/events/${encodeURIComponent(eventId)}`, {
    credentials: "include",
  });
  const payload = (await response.json().catch(() => null)) as
    (EventDetailResult & { message?: string }) | { message?: string } | null;

  if (!response.ok) {
    throw new Error(readErrorMessage(payload, "Event could not be loaded."));
  }

  if (!payload || !("event" in payload)) {
    throw new Error("The event detail response was not recognized.");
  }

  return payload;
}

export async function fetchDashboardEvents(): Promise<DashboardEventsResult> {
  const response = await fetch("/api/events/dashboard", {
    credentials: "include",
  });
  const payload = (await response.json().catch(() => null)) as
    (DashboardEventsResult & { message?: string }) | { message?: string } | null;

  if (!response.ok) {
    throw new Error(readErrorMessage(payload, "Dashboard events could not be loaded."));
  }

  if (!payload || !("upcoming" in payload) || !("past" in payload)) {
    throw new Error("The dashboard events response was not recognized.");
  }

  return payload;
}

export async function fetchEventRsvps(eventId: string): Promise<EventRsvpListResult> {
  const response = await fetch(`/api/events/${encodeURIComponent(eventId)}/rsvps`, {
    credentials: "include",
  });
  const payload = (await response.json().catch(() => null)) as
    (EventRsvpListResult & { message?: string }) | { message?: string } | null;

  if (!response.ok) {
    throw new Error(readErrorMessage(payload, "RSVP list could not be loaded."));
  }

  if (
    !payload ||
    !("coming" in payload) ||
    !("declined" in payload) ||
    !("maybe" in payload)
  ) {
    throw new Error("The RSVP list response was not recognized.");
  }

  return payload;
}

export async function fetchEventComments(eventId: string): Promise<EventCommentsResult> {
  const response = await fetch(`/api/events/${encodeURIComponent(eventId)}/comments`, {
    credentials: "include",
  });
  const payload = (await response.json().catch(() => null)) as
    (EventCommentsResult & { message?: string }) | { message?: string } | null;

  if (!response.ok) {
    throw new Error(readErrorMessage(payload, "Comments could not be loaded."));
  }

  if (!payload || !("comments" in payload)) {
    throw new Error("The comments response was not recognized.");
  }

  return payload;
}

export async function createEventComment(
  eventId: string,
  body: string,
): Promise<CreateCommentResult> {
  const response = await fetch(`/api/events/${encodeURIComponent(eventId)}/comments`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ body }),
  });
  const payload = (await response.json().catch(() => null)) as
    (CreateCommentResult & { message?: string }) | { message?: string } | null;

  if (!response.ok) {
    throw new Error(readErrorMessage(payload, "Comment could not be posted."));
  }

  if (!payload || !("comment" in payload)) {
    throw new Error("The comment response was not recognized.");
  }

  return payload;
}

export async function deleteEventComment(
  eventId: string,
  commentId: string,
): Promise<void> {
  const response = await fetch(
    `/api/events/${encodeURIComponent(eventId)}/comments/${encodeURIComponent(commentId)}`,
    {
      method: "DELETE",
      credentials: "include",
    },
  );
  const payload = (await response.json().catch(() => null)) as
    { message?: string } | null;

  if (!response.ok) {
    throw new Error(readErrorMessage(payload, "Comment could not be deleted."));
  }
}

export async function inviteFriendByEmail(
  eventId: string,
  email: string,
): Promise<InviteFriendResult> {
  return createInvitation(
    eventId,
    email,
    "invitations",
    "Invitation could not be sent.",
  );
}

export async function createShareableInviteLink(
  eventId: string,
  email: string,
): Promise<InviteFriendResult> {
  return createInvitation(
    eventId,
    email,
    "invitations/share-link",
    "Invitation link could not be created.",
  );
}

async function createInvitation(
  eventId: string,
  email: string,
  path: "invitations" | "invitations/share-link",
  fallback: string,
): Promise<InviteFriendResult> {
  const response = await fetch(`/api/events/${encodeURIComponent(eventId)}/${path}`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ email }),
  });
  const payload = (await response.json().catch(() => null)) as
    (InviteFriendResult & { message?: string }) | { message?: string } | null;

  if (!response.ok) {
    throw new Error(readErrorMessage(payload, fallback));
  }

  if (!payload || !("invitation" in payload) || !("invitation_url" in payload)) {
    throw new Error("The invitation response was not recognized.");
  }

  return payload;
}

export async function updateInvitationRsvp(
  shareToken: string,
  status: RsvpStatus,
): Promise<RsvpUpdateResult> {
  const response = await fetch(
    `/api/invitations/${encodeURIComponent(shareToken)}/rsvp`,
    {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ status }),
    },
  );
  const payload = (await response.json().catch(() => null)) as
    (RsvpUpdateResult & { message?: string }) | { message?: string } | null;

  if (!response.ok) {
    throw new Error(readErrorMessage(payload, "RSVP could not be updated."));
  }

  if (!payload || !("invitation" in payload) || !("event" in payload)) {
    throw new Error("The RSVP response was not recognized.");
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
