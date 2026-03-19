import { writable } from 'svelte/store';
import type { User } from '$lib/types';

interface AuthState {
  user: User | null;
  loading: boolean;
}

export const auth = writable<AuthState>({ user: null, loading: true });
export const theme = writable<'light' | 'dark'>('dark');
