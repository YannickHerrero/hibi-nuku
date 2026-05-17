// Bearer token storage + setter.
//
// We treat the token as a per-device secret entered once in the setup
// wizard. localStorage is good enough for personal-use over Tailscale.

const KEY = "nuku:token";

export function getToken(): string | null {
  return localStorage.getItem(KEY);
}

export function setToken(t: string) {
  localStorage.setItem(KEY, t);
}

export function clearToken() {
  localStorage.removeItem(KEY);
}
