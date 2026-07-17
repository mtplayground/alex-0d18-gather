export type AuthStartResponse = {
  status: string;
  auth_url: string;
  email_sent?: boolean;
};

export async function startRegistration(email: string): Promise<AuthStartResponse> {
  return postAuth("/api/auth/register", {
    email,
    return_to: "/",
  });
}

export async function startLogin(email: string): Promise<AuthStartResponse> {
  return postAuth("/api/auth/login", {
    email,
    return_to: "/",
  });
}

export function googleAuthUrl(returnTo = "/"): string {
  return `/api/auth/google?return_to=${encodeURIComponent(returnTo)}`;
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
