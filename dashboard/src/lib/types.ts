export interface SummaryGroup {
  name: string;
  seconds: number;
  time: string;
  pct: number;
}

export interface SummaryResponse {
  period_from: string;
  period_to: string;
  total_active: string;
  total_active_seconds: number;
  groups: SummaryGroup[];
}

export interface AppEntry {
  app: string;
  category: string;
  seconds: number;
  time: string;
  pct: number;
  events: number;
}

export interface TimelineEntry {
  timestamp: string;
  duration: number;
  app: string | null;
  title: string | null;
  url: string | null;
  category: string;
  is_afk: boolean;
}

export interface ProductivityData {
  score: number;
  productive: number;
  neutral: number;
  distracting: number;
  total: number;
}

export interface TrendBucket {
  bucket: string;
  total_seconds: number;
  total_hours: number;
  productivity: number;
  categories: Record<string, number>;
}

export interface CurrentActivity {
  app: string | null;
  title: string | null;
  url: string | null;
  category: string;
  duration_seconds: number;
  is_afk: boolean;
  since: string;
}

export interface Category {
  id: number;
  name: string;
  parent_id: number | null;
  productivity_score: number;
}

export interface DateRange {
  from: string;
  to: string;
  label: string;
}

export interface RuleCreateRequest {
  app: string;
  category_id: number;
  field: "app" | "title" | "url_domain";
}

export interface AppDetailsResponse {
  app: string;
  sessions: AppSession[];
}

export interface AppSession {
  timestamp: string;
  duration: number;
  app: string | null;
  title: string | null;
  url: string | null;
  category: string;
}

export interface CategoryRule {
  id: number;
  category_id: number;
  category_name: string;
  field: "app" | "title" | "url_domain";
  pattern: string;
  is_builtin: boolean;
  priority: number;
}
