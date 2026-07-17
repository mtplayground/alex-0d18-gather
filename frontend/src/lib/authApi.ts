export type AuthStartResponse = {
  status: string;
  auth_url: string;
  email_sent?: boolean;
};

export type AuthUser = {
  id: string;
  email: string;
  display_name: string | null;
  full_name: string | null;
  avatar_object_key: string | null;
  email_verified: boolean;
};

export async function fetchCurrentUser(): Promise<AuthUser | null> {
  const response = await fetch("/api/auth/me", {
    credentials: "include",
  });

  if (response.status === 401 || response.status === 403) {
    return null;
  }

  if (!response.ok) {
    throw new Error("Session status could not be loaded.");
  }

  return (await response.json()) as AuthUser;
}

export async function startRegistration(
  email: string,
  returnTo = "/",
): Promise<AuthStartResponse> {
  return postAuth("/api/auth/register", {
    email,
    return_to: normalizeReturnTo(returnTo),
  });
}

export async function startLogin(
  email: string,
  returnTo = "/",
): Promise<AuthStartResponse> {
  return postAuth("/api/auth/login", {
    email,
    return_to: normalizeReturnTo(returnTo),
  });
}

export function googleAuthUrl(returnTo = "/"): string {
  return `/api/auth/google?return_to=${encodeURIComponent(normalizeReturnTo(returnTo))}`;
}

async function postAuth(
  path: string,
  body: Record<string, string>,
): Promise<AuthStartResponse> {
  const response = await fetch(path, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(body),
  });
  const payload = (await response.json().catch(() => null)) as
    AuthStartResponse | { message?: string } | null;

  if (!response.ok) {
    throw new Error(
      payload && "message" in payload && payload.message
        ? payload.message
        : "The auth request could not be completed.",
    );
  }

  if (!payload || !("auth_url" in payload)) {
    throw new Error("The auth service returned an unexpected response.");
  }

  return payload;
}

export function normalizeReturnTo(value: string | null | undefined): string {
  if (!value || !value.startsWith("/") || value.startsWith("/api/")) {
    return "/";
  }

  return value;
}
