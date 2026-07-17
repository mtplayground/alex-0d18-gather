import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import InvitationRsvpPage from "../src/pages/InvitationRsvpPage";
import { updateInvitationRsvp } from "../src/lib/eventsApi";

vi.mock("../src/lib/eventsApi", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../src/lib/eventsApi")>();
  return {
    ...actual,
    updateInvitationRsvp: vi.fn(),
  };
});

describe("InvitationRsvpPage", () => {
  beforeEach(() => {
    vi.mocked(updateInvitationRsvp).mockReset();
  });

  it("saves a one-tap RSVP and renders the confirmation details", async () => {
    vi.mocked(updateInvitationRsvp).mockResolvedValue({
      status: "rsvp_updated",
      email_sent: true,
      invitation: {
        id: "invitation-1",
        event_id: "event-123",
        invitee_user_id: "user-2",
        invitee_email: "guest@example.com",
        status: "accepted",
        share_token: "share-token",
        rsvp_status: "maybe",
        rsvp_responded_at: "2026-07-17T08:00:00Z",
        created_at: "2026-07-17T08:00:00Z",
        updated_at: "2026-07-17T08:00:00Z",
      },
      event: {
        id: "event-123",
        organizer_user_id: "user-1",
        title: "Summer dinner",
        description: "Bring a side.",
        starts_at: "2026-08-01T23:00:00Z",
        ends_at: null,
        timezone: "UTC",
        location_name: "Garden",
        location_address: "42 Bloom St",
        cover_image_object_key: null,
        pdf_attachment_object_keys: [],
        created_at: "2026-07-17T08:00:00Z",
        updated_at: "2026-07-17T08:00:00Z",
      },
    });

    render(<InvitationRsvpPage shareToken="share-token" />);

    await userEvent.click(screen.getByRole("button", { name: "Maybe" }));

    await waitFor(() => {
      expect(updateInvitationRsvp).toHaveBeenCalledWith("share-token", "maybe");
    });
    expect(
      await screen.findByText(
        "Your RSVP is saved and a confirmation email is on its way.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByText("Current response: Maybe")).toBeInTheDocument();
    expect(screen.getByText("Summer dinner")).toBeInTheDocument();
    expect(screen.getByText("Garden · 42 Bloom St")).toBeInTheDocument();
  });
});
