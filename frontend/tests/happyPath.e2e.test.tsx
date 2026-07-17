import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import EventCreatePage from "../src/pages/EventCreatePage";
import EventDetailPage from "../src/pages/EventDetailPage";
import InvitationRsvpPage from "../src/pages/InvitationRsvpPage";
import {
  EventActivityEntry,
  EventComment,
  EventRecord,
  InvitationRecord,
  RsvpListEntry,
} from "../src/lib/eventsApi";

const authState = vi.hoisted(() => ({
  user: {
    id: "host-user",
    email: "host@example.com",
    display_name: "Host",
    full_name: null,
    bio: null,
    location: null,
    website_url: null,
    avatar_object_key: null,
    avatar_url: null,
    email_verified: true,
  },
}));

vi.mock("../src/auth/useAuth", () => ({
  useAuth: () => ({
    status: "authenticated",
    user: authState.user,
    refresh: vi.fn(),
  }),
}));

describe("happy path E2E flow", () => {
  let backend: FakeBackend;

  beforeEach(() => {
    backend = createFakeBackend();
    authState.user = backend.hostUser;
    vi.stubGlobal("fetch", vi.fn(backend.fetch));
  });

  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
  });

  it("lets a host create an event, invite a guest, then records the guest RSVP and comment", async () => {
    const user = userEvent.setup();

    render(<EventCreatePage />);

    await user.type(screen.getByLabelText("Event title"), "Backyard brunch");
    await user.type(screen.getByLabelText("Description"), "Bring a favorite dish.");
    await user.type(screen.getByLabelText("Starts"), "2026-08-01T11:00");
    await user.clear(screen.getByLabelText("Timezone"));
    await user.type(screen.getByLabelText("Timezone"), "UTC");
    await user.type(screen.getByLabelText("Location name"), "Maple Yard");
    await user.type(screen.getByLabelText("Location address"), "10 Garden Lane");
    await user.click(screen.getByRole("button", { name: "Create event" }));

    expect(await screen.findByText("Event created.")).toBeInTheDocument();
    expect(backend.event?.title).toBe("Backyard brunch");
    expect(screen.getByRole("link", { name: "Open event detail" })).toHaveAttribute(
      "href",
      "/events/event-1",
    );

    cleanup();
    render(<EventDetailPage eventId="event-1" />);

    expect(
      await screen.findByRole("heading", { name: "Backyard brunch" }),
    ).toBeInTheDocument();
    await user.type(screen.getByLabelText("Email address"), "guest@example.com");
    await user.click(screen.getByRole("button", { name: "Send invite" }));

    expect(
      await screen.findByText("Invitation sent to guest@example.com."),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Shareable link")).toHaveValue(
      "/invitations/share-token-1",
    );

    authState.user = backend.guestUser;
    cleanup();
    render(<InvitationRsvpPage shareToken="share-token-1" />);

    await user.click(screen.getByRole("button", { name: "Yes" }));

    expect(
      await screen.findByText(
        "Your RSVP is saved and a confirmation email is on its way.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByText("Current response: Yes")).toBeInTheDocument();
    expect(screen.getByText("Backyard brunch")).toBeInTheDocument();

    cleanup();
    render(<EventDetailPage eventId="event-1" />);

    expect(await screen.findByText("guest@example.com")).toBeInTheDocument();
    await user.type(
      screen.getByLabelText("Add a comment"),
      "Looking forward to it!",
    );
    await user.click(screen.getByRole("button", { name: "Post comment" }));

    expect(
      await screen.findByText("Looking forward to it!"),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(backend.comments).toHaveLength(1);
      expect(backend.comments[0].author_user_id).toBe("guest-user");
      expect(backend.activity.map((entry) => entry.activity_type)).toContain(
        "comment_created",
      );
    });
  });
});

type FakeUser = typeof authState.user;

type FakeBackend = ReturnType<typeof createFakeBackend>;

function createFakeBackend() {
  const hostUser: FakeUser = {
    ...authState.user,
    id: "host-user",
    email: "host@example.com",
    display_name: "Host",
  };
  const guestUser: FakeUser = {
    ...authState.user,
    id: "guest-user",
    email: "guest@example.com",
    display_name: "Guest",
  };
  const activity: EventActivityEntry[] = [];
  const comments: EventComment[] = [];
  let event: EventRecord | null = null;
  let invitation: InvitationRecord | null = null;

  async function fetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
    const path = typeof input === "string" ? input : input.toString();
    const method = init?.method ?? "GET";

    if (path === "/api/events/" && method === "POST") {
      const form = init?.body as FormData;
      event = {
        id: "event-1",
        organizer_user_id: hostUser.id,
        title: String(form.get("title") ?? ""),
        description: formValue(form, "description"),
        starts_at: String(form.get("starts_at") ?? ""),
        ends_at: formValue(form, "ends_at"),
        timezone: formValue(form, "timezone"),
        location_name: formValue(form, "location_name"),
        location_address: formValue(form, "location_address"),
        cover_image_object_key: null,
        pdf_attachment_object_keys: [],
        created_at: "2026-07-17T09:00:00Z",
        updated_at: "2026-07-17T09:00:00Z",
      };
      activity.push(activityEntry("activity-1", "event_created", "Event created"));
      return json({ status: "created", event }, 201);
    }

    if (path === "/api/events/event-1" && method === "GET") {
      return json({
        event: {
          event: requireEvent(event),
          viewer_role: authState.user.id === hostUser.id ? "host" : "guest",
          cover_image: null,
          pdf_attachments: [],
        },
      });
    }

    if (path === "/api/events/event-1/invitations" && method === "POST") {
      const body = parseJsonBody(init);
      invitation = {
        id: "invitation-1",
        event_id: "event-1",
        invitee_user_id: guestUser.id,
        invitee_email: String(body.email),
        status: "pending",
        share_token: "share-token-1",
        rsvp_status: null,
        rsvp_responded_at: null,
        created_at: "2026-07-17T09:01:00Z",
        updated_at: "2026-07-17T09:01:00Z",
      };
      return json({
        status: "invitation_created",
        invitation,
        invitation_url: "/invitations/share-token-1",
        email_sent: true,
      });
    }

    if (path === "/api/events/event-1/comments" && method === "GET") {
      return json({ comments });
    }

    if (path === "/api/events/event-1/comments" && method === "POST") {
      const body = parseJsonBody(init);
      const comment: EventComment = {
        id: `comment-${comments.length + 1}`,
        event_id: "event-1",
        author_user_id: authState.user.id,
        body: String(body.body),
        created_at: "2026-07-17T09:03:00Z",
        updated_at: "2026-07-17T09:03:00Z",
      };
      comments.push(comment);
      activity.push(activityEntry("activity-3", "comment_created", "Comment added"));
      return json({ status: "comment_created", comment }, 201);
    }

    if (path === "/api/events/event-1/timeline" && method === "GET") {
      return json({ activity });
    }

    if (path === "/api/events/event-1/rsvps" && method === "GET") {
      return json({
        event_id: "event-1",
        coming: invitation?.rsvp_status === "yes" ? [rsvpEntry(invitation)] : [],
        declined: invitation?.rsvp_status === "no" ? [rsvpEntry(invitation)] : [],
        maybe: invitation?.rsvp_status === "maybe" ? [rsvpEntry(invitation)] : [],
      });
    }

    if (path === "/api/invitations/share-token-1/rsvp" && method === "POST") {
      const currentInvitation = requireInvitation(invitation);
      const body = parseJsonBody(init);
      currentInvitation.rsvp_status = String(body.status);
      currentInvitation.rsvp_responded_at = "2026-07-17T09:02:00Z";
      currentInvitation.updated_at = "2026-07-17T09:02:00Z";
      activity.push(activityEntry("activity-2", "rsvp_updated", "Guest RSVP updated"));
      return json({
        status: "rsvp_updated",
        invitation: currentInvitation,
        event: requireEvent(event),
        email_sent: true,
      });
    }

    return json({ message: `Unhandled ${method} ${path}` }, 404);
  }

  return {
    activity,
    comments,
    fetch,
    get event() {
      return event;
    },
    get invitation() {
      return invitation;
    },
    guestUser,
    hostUser,
  };
}

function formValue(form: FormData, key: string): string | null {
  const value = form.get(key);
  return value ? String(value) : null;
}

function parseJsonBody(init: RequestInit | undefined): Record<string, unknown> {
  return JSON.parse(String(init?.body ?? "{}")) as Record<string, unknown>;
}

function requireEvent(event: EventRecord | null): EventRecord {
  if (!event) {
    throw new Error("Expected fake event to exist.");
  }
  return event;
}

function requireInvitation(invitation: InvitationRecord | null): InvitationRecord {
  if (!invitation) {
    throw new Error("Expected fake invitation to exist.");
  }
  return invitation;
}

function rsvpEntry(invitation: InvitationRecord): RsvpListEntry {
  return {
    invitation_id: invitation.id,
    invitee_user_id: invitation.invitee_user_id,
    invitee_email: invitation.invitee_email,
    rsvp_status: invitation.rsvp_status as RsvpListEntry["rsvp_status"],
    rsvp_responded_at: invitation.rsvp_responded_at ?? "2026-07-17T09:02:00Z",
  };
}

function activityEntry(
  id: string,
  activityType: string,
  message: string,
): EventActivityEntry {
  return {
    id,
    event_id: "event-1",
    actor_user_id: authState.user.id,
    activity_type: activityType,
    subject_type: null,
    subject_id: null,
    message,
    created_at: "2026-07-17T09:00:00Z",
  };
}

function json(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      "Content-Type": "application/json",
    },
  });
}
