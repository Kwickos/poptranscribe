export interface Session {
  id: string;
  title: string;
  mode: string;
  audio_path: string | null;
  created_at: string;
  duration_secs: number | null;
  summary_json: string | null;
}

export interface Segment {
  id: number;
  session_id: string;
  text: string;
  start_time: number;
  end_time: number;
  speaker: string | null;
  is_diarized: boolean;
}

export interface Summary {
  key_points: string[];
  decisions: string[];
  action_items: ActionItem[];
}

export interface ActionItem {
  description: string;
  assignee: string | null;
}

export interface SessionDetail extends Session {
  segments: Segment[];
  summary: Summary | null;
}
