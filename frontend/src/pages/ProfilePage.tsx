import { FormEvent, useEffect, useMemo, useState } from "react";

import { useAuth } from "../auth/useAuth";
import {
  requestPasswordReset,
  updateProfile,
  uploadProfilePhoto,
} from "../lib/authApi";

type FormState = {
  display_name: string;
  full_name: string;
  bio: string;
  location: string;
  website_url: string;
};

type SaveState = "idle" | "saving" | "success" | "error";

function ProfilePage() {
  const { user, refresh } = useAuth();
  const [form, setForm] = useState<FormState>(emptyForm);
  const [saveState, setSaveState] = useState<SaveState>("idle");
  const [message, setMessage] = useState("");
  const [photoFile, setPhotoFile] = useState<File | null>(null);
  const [photoState, setPhotoState] = useState<SaveState>("idle");
  const [passwordState, setPasswordState] = useState<SaveState>("idle");
  const [passwordMessage, setPasswordMessage] = useState("");
  const [passwordAuthUrl, setPasswordAuthUrl] = useState("");

  const initials = useMemo(
    () => initialsFor(user?.display_name || user?.full_name || user?.email),
    [user],
  );

  useEffect(() => {
    if (!user) {
      return;
    }

    setForm({
      display_name: user.display_name ?? "",
      full_name: user.full_name ?? "",
      bio: user.bio ?? "",
      location: user.location ?? "",
      website_url: user.website_url ?? "",
    });
  }, [user]);

  async function saveProfile(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSaveState("saving");
    setMessage("");

    try {
      await updateProfile({
        display_name: blankToNull(form.display_name),
        full_name: blankToNull(form.full_name),
        bio: blankToNull(form.bio),
        location: blankToNull(form.location),
        website_url: blankToNull(form.website_url),
      });
      await refresh();
      setSaveState("success");
      setMessage("Profile saved.");
    } catch (error) {
      setSaveState("error");
      setMessage(
        error instanceof Error ? error.message : "Profile could not be saved.",
      );
    }
  }

  async function uploadPhoto() {
    if (!photoFile) {
      return;
    }

    setPhotoState("saving");
    setMessage("");

    try {
      await uploadProfilePhoto(photoFile);
      await refresh();
      setPhotoState("success");
      setPhotoFile(null);
      setMessage("Profile photo updated.");
    } catch (error) {
      setPhotoState("error");
      setMessage(
        error instanceof Error ? error.message : "Profile photo could not be uploaded.",
      );
    }
  }

  async function sendPasswordReset() {
    if (!user?.email) {
      return;
    }

    setPasswordState("saving");
    setPasswordMessage("");
    setPasswordAuthUrl("");

    try {
      const result = await requestPasswordReset(user.email, "/profile");
      setPasswordState("success");
      setPasswordAuthUrl(result.email_sent ? "" : result.auth_url);
      setPasswordMessage(
        result.email_sent
          ? "Password change link sent to your email."
          : "Email delivery is not configured here. Continue with secure auth instead.",
      );
    } catch (error) {
      setPasswordState("error");
      setPasswordMessage(
        error instanceof Error
          ? error.message
          : "Password change could not be started.",
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
              Profile settings
            </h1>
            <p className="mt-3 max-w-2xl text-base leading-7 text-slate-700">
              Keep your account details current for invitations, RSVP updates, and event
              planning.
            </p>
          </div>
          <a
            className="inline-flex h-11 items-center justify-center rounded-md border border-slate-300 bg-white px-4 text-sm font-semibold text-slate-900 transition hover:border-slate-500 hover:bg-slate-100"
            href="/dashboard"
          >
            Back to dashboard
          </a>
        </header>

        <div className="mt-8 grid gap-6 lg:grid-cols-[360px_1fr]">
          <aside className="space-y-6">
            <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
              <div className="flex items-center gap-4">
                {user?.avatar_url ? (
                  <img
                    alt=""
                    className="h-20 w-20 rounded-full border border-slate-200 object-cover"
                    src={user.avatar_url}
                  />
                ) : (
                  <div className="grid h-20 w-20 place-items-center rounded-full bg-emerald-100 text-2xl font-semibold text-emerald-800">
                    {initials}
                  </div>
                )}
                <div>
                  <p className="font-semibold text-slate-950">
                    {user?.display_name || user?.full_name || "Your profile"}
                  </p>
                  <p className="mt-1 text-sm text-slate-600">{user?.email}</p>
                </div>
              </div>

              <div className="mt-6 space-y-3">
                <label className="block">
                  <span className="text-sm font-medium text-slate-800">
                    Profile photo
                  </span>
                  <input
                    accept="image/jpeg,image/png,image/webp,image/gif"
                    className="mt-2 block w-full text-sm text-slate-700 file:mr-4 file:rounded-md file:border-0 file:bg-slate-900 file:px-4 file:py-2 file:text-sm file:font-semibold file:text-white"
                    onChange={(event) => setPhotoFile(event.target.files?.[0] ?? null)}
                    type="file"
                  />
                </label>
                {photoFile ? (
                  <p className="text-sm text-slate-600">Selected: {photoFile.name}</p>
                ) : null}
                <button
                  className="h-11 w-full rounded-md bg-emerald-700 px-4 text-sm font-semibold text-white transition hover:bg-emerald-800 disabled:cursor-not-allowed disabled:bg-slate-400"
                  disabled={!photoFile || photoState === "saving"}
                  onClick={uploadPhoto}
                  type="button"
                >
                  {photoState === "saving" ? "Uploading..." : "Upload photo"}
                </button>
              </div>
            </section>

            <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
              <h2 className="text-base font-semibold text-slate-950">Account</h2>
              <div className="mt-4 space-y-4">
                <div>
                  <p className="text-xs font-semibold uppercase text-slate-500">
                    Email
                  </p>
                  <p className="mt-1 text-sm font-semibold text-slate-950">
                    {user?.email}
                  </p>
                  <p className="mt-1 text-sm text-slate-600">
                    {user?.email_verified ? "Verified" : "Verification pending"}
                  </p>
                </div>
                <button
                  className="h-11 w-full rounded-md border border-slate-300 bg-white px-4 text-sm font-semibold text-slate-900 transition hover:border-slate-500 hover:bg-slate-100 disabled:cursor-not-allowed disabled:bg-slate-100"
                  disabled={passwordState === "saving"}
                  onClick={sendPasswordReset}
                  type="button"
                >
                  {passwordState === "saving"
                    ? "Sending..."
                    : "Send password change link"}
                </button>
                {passwordMessage ? (
                  <div>
                    <p
                      className={`text-sm ${
                        passwordState === "error" ? "text-rose-700" : "text-emerald-700"
                      }`}
                    >
                      {passwordMessage}
                    </p>
                    {passwordAuthUrl ? (
                      <a
                        className="mt-2 inline-block text-sm font-semibold text-emerald-700"
                        href={passwordAuthUrl}
                      >
                        Continue with secure auth
                      </a>
                    ) : null}
                  </div>
                ) : null}
              </div>
            </section>
          </aside>

          <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
            <h2 className="text-base font-semibold text-slate-950">Profile details</h2>
            <form className="mt-5 grid gap-5" onSubmit={saveProfile}>
              <ProfileInput
                label="Display name"
                onChange={(value) =>
                  setForm((current) => ({ ...current, display_name: value }))
                }
                value={form.display_name}
              />
              <ProfileInput
                label="Full name"
                onChange={(value) =>
                  setForm((current) => ({ ...current, full_name: value }))
                }
                value={form.full_name}
              />
              <label className="block">
                <span className="text-sm font-medium text-slate-800">Bio</span>
                <textarea
                  className="mt-2 min-h-32 w-full rounded-md border border-slate-300 bg-white px-4 py-3 text-base outline-none transition focus:border-emerald-600 focus:ring-4 focus:ring-emerald-100"
                  maxLength={500}
                  onChange={(event) =>
                    setForm((current) => ({ ...current, bio: event.target.value }))
                  }
                  value={form.bio}
                />
              </label>
              <ProfileInput
                label="Location"
                onChange={(value) =>
                  setForm((current) => ({ ...current, location: value }))
                }
                value={form.location}
              />
              <ProfileInput
                label="Website"
                onChange={(value) =>
                  setForm((current) => ({ ...current, website_url: value }))
                }
                placeholder="https://example.com"
                type="url"
                value={form.website_url}
              />

              {message ? (
                <p
                  className={`text-sm ${
                    saveState === "error" || photoState === "error"
                      ? "text-rose-700"
                      : "text-emerald-700"
                  }`}
                >
                  {message}
                </p>
              ) : null}

              <button
                className="h-12 rounded-md bg-emerald-700 px-5 text-base font-semibold text-white transition hover:bg-emerald-800 disabled:cursor-not-allowed disabled:bg-slate-400 sm:w-fit"
                disabled={saveState === "saving"}
                type="submit"
              >
                {saveState === "saving" ? "Saving..." : "Save profile"}
              </button>
            </form>
          </section>
        </div>
      </section>
    </main>
  );
}

function ProfileInput({
  label,
  onChange,
  placeholder,
  type = "text",
  value,
}: {
  label: string;
  onChange: (value: string) => void;
  placeholder?: string;
  type?: "text" | "url";
  value: string;
}) {
  return (
    <label className="block">
      <span className="text-sm font-medium text-slate-800">{label}</span>
      <input
        className="mt-2 h-12 w-full rounded-md border border-slate-300 bg-white px-4 text-base outline-none transition focus:border-emerald-600 focus:ring-4 focus:ring-emerald-100"
        maxLength={type === "url" ? 2048 : 120}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
        type={type}
        value={value}
      />
    </label>
  );
}

const emptyForm: FormState = {
  display_name: "",
  full_name: "",
  bio: "",
  location: "",
  website_url: "",
};

function blankToNull(value: string): string | null {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
}

function initialsFor(value: string | undefined): string {
  if (!value) {
    return "G";
  }

  const parts = value.trim().split(/\s+/).filter(Boolean).slice(0, 2);

  return parts.map((part) => part[0]?.toUpperCase()).join("") || "G";
}

export default ProfilePage;
