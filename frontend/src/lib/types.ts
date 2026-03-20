export interface User {
  id: string;
  username: string;
  email: string;
  perms: number;
  created_at: string | null;
  avatar_url: string | null;
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
  share_token: string | null;
}

export interface EventMember {
  event_id: number;
  user_id: string;
  username: string | null;
  avatar_url: string | null;
  status: 'going' | 'late' | 'not_going';
  late_minutes: number | null;
  joined_at: string | null;
}

export type RsvpStatus = 'going' | 'late' | 'not_going';

export type PollType = 'choice' | 'text' | 'rating' | 'yesno' | 'date';

export interface PollChoice {
  id: number;
  label: string;
  position: number;
  answer_count: number;
}

export interface Poll {
  id: number;
  event_id: number;
  question: string;
  poll_type: PollType;
  allow_multiple: boolean;
  choices: PollChoice[];
  my_choice_ids: number[];
  my_text_answer: string | null;
  my_rating: number | null;
  avg_rating: number | null;
  rating_count: number;
}

export interface VoterEntry {
  user_id: string;
  username: string;
}

export interface ChoiceVoters {
  choice_id: number;
  label: string;
  voters: VoterEntry[];
}

export interface TextAnswer {
  user_id: string;
  username: string;
  answer: string;
}

export interface PollTemplate {
  id: number;
  name: string;
  poll_type: PollType;
  question: string | null;
  choices: { label: string }[] | null;
  allow_multiple: boolean;
  global: boolean;
  created_by: string | null;
}