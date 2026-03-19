export interface User {
  id: string;
  username: string;
  email: string;
  perms: number;
  created_at: string | null;
}

export interface Event {
  id: number;
  title: string;
  description: string | null;
  date: string;
  location: string | null;
  color: string | null;
  created_by: string;
  created_at: string | null;
  private: boolean;
  creator_name: string | null;
}

export interface EventMember {
  event_id: number;
  user_id: string;
  username: string | null;
  status: 'going' | 'late' | 'not_going';
  late_minutes: number | null;
  joined_at: string | null;
}

export type RsvpStatus = 'going' | 'late' | 'not_going';
