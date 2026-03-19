import { browser } from '$app/environment';
import { goto } from '$app/navigation';
import { auth } from '$lib/stores';

const BASE = import.meta.env.PUBLIC_API_URL ?? 'http://localhost:3001';

let _accessToken: string | null = null;
let _refreshToken: string | null = null;

if (browser) {
  _accessToken  = localStorage.getItem('access_token');
  _refreshToken = localStorage.getItem('refresh_token');
}

export function setTokens(access: string, refresh: string) {
  _accessToken  = access;
  _refreshToken = refresh;
  if (browser) {
    localStorage.setItem('access_token',  access);
    localStorage.setItem('refresh_token', refresh);
  }
}

export function clearTokens() {
  _accessToken  = null;
  _refreshToken = null;
  if (browser) {
    localStorage.removeItem('access_token');
    localStorage.removeItem('refresh_token');
  }
}

async function refreshAccessToken(): Promise<boolean> {
  if (!_refreshToken) return false;
  try {
    const res = await fetch(`${BASE}/api/auth/refresh`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refresh_token: _refreshToken })
    });
    if (!res.ok) return false;
    const data = await res.json();
    _accessToken = data.access_token;
    if (browser) localStorage.setItem('access_token', data.access_token);
    return true;
  } catch {
    return false;
  }
}

export async function apiFetch<T = unknown>(
  path: string,
  options: RequestInit = {}
): Promise<T> {
  const doRequest = async (token: string | null) => {
    return fetch(`${BASE}${path}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
        ...(options.headers ?? {})
      }
    });
  };

  let res = await doRequest(_accessToken);

  // If 401, try to refresh once
  if (res.status === 401) {
    const refreshed = await refreshAccessToken();
    if (refreshed) {
      res = await doRequest(_accessToken);
    } else {
      clearTokens();
      auth.set({ user: null, loading: false });
      goto('/login');
      throw new Error('Unauthorized');
    }
  }

  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: 'Request failed' }));
    throw new Error(err.error ?? 'Request failed');
  }

  return res.json() as Promise<T>;
}
