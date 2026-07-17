import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import EventCreatePage from "../src/pages/EventCreatePage";
import { createEvent } from "../src/lib/eventsApi";

const authState = vi.hoisted(() => ({
  status: "authenticated",
  user: {
    id: "user-1",
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
  useAuth: () => authState,
}));

vi.mock("../src/lib/eventsApi", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../src/lib/eventsApi")>();
  return {
    ...actual,
    createEvent: vi.fn(),
  };
});

describe("EventCreatePage", () => {
  beforeEach(() => {
    vi.mocked(createEvent).mockReset();
  });

  it("submits event details and shows the created event link", async () => {
    vi.mocked(createEvent).mockResolvedValue({
      status: "created",
      event: {
        id: "event-123",
        organizer_user_id: "user-1",
        title: "Summer dinner",
        description: "Bring a side.",
        starts_at: "2026-08-01T23:00:00Z",
        ends_at: "2026-08-02T02:00:00Z",
        timezone: "UTC",
        location_name: "Garden",
        location_address: "42 Bloom St",
        cover_image_object_key: null,
        pdf_attachment_object_keys: [],
        created_at: "2026-07-17T08:00:00Z",
        updated_at: "2026-07-17T08:00:00Z",
      },
    });

    render(<EventCreatePage />);

    await userEvent.type(screen.getByLabelText("Event title"), "Summer dinner");
    await userEvent.type(screen.getByLabelText("Description"), "Bring a side.");
    await userEvent.type(screen.getByLabelText("Starts"), "2026-08-01T18:00");
    await userEvent.type(screen.getByLabelText("Ends"), "2026-08-01T21:00");
    await userEvent.clear(screen.getByLabelText("Timezone"));
    await userEvent.type(screen.getByLabelText("Timezone"), "UTC");
    await userEvent.type(screen.getByLabelText("Location name"), "Garden");
    await userEvent.type(screen.getByLabelText("Location address"), "42 Bloom St");
    await userEvent.click(screen.getByRole("button", { name: "Create event" }));

    await waitFor(() => {
      expect(createEvent).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Summer dinner",
          description: "Bring a side.",
          startsAt: "2026-08-01T18:00",
          endsAt: "2026-08-01T21:00",
          timezone: "UTC",
          locationName: "Garden",
          locationAddress: "42 Bloom St",
          coverImage: null,
          pdfAttachments: [],
        }),
      );
    });
    expect(await screen.findByText("Event created.")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Open event detail" })).toHaveAttribute(
      "href",
      "/events/event-123",
    );
  });
});
