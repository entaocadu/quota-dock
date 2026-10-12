export type ProviderId =
  "openai-codex" | "google-antigravity" | "opencode-local";

export type OriginKind =
  | "opencode_openai_usage"
  | "codex_local_usage"
  | "openai_quota_login"
  | "codex_app_server"
  | "codex_web_manual"
  | "openai_undocumented_opt_in"
  | "antigravity_plugin_cache"
  | "antigravity_private_opt_in"
  | "opencode_stats";

export type Availability =
  | "ok"
  | "partial"
  | "stale"
  | "disabled"
  | "not_configured"
  | "not_installed"
  | "unauthorized_scope"
  | "forbidden"
  | "rate_limited"
  | "offline"
  | "network_error"
  | "source_changed"
  | "no_data"
  | "unknown_error";

export interface QuotaSnapshot {
  schemaVersion: number;
  providerId: ProviderId;
  origin: OriginKind;
  accountKey: string;
  accountLabel: string;
  status: Availability;
  observedAt: string | null;
  attemptedAt: string | null;
  expiresAt: string | null;
  sourceVersion?: string | null;
  note?: string | null;
  windows: QuotaWindow[];
  models: ModelQuota[];
  warnings: string[];
  rawErrorCode?: string | null;
}

export interface QuotaWindow {
  id: string;
  label: string;
  usedPercent: number | null;
  remainingPercent: number | null;
  windowSeconds: number | null;
  resetAt: string | null;
  allowed: boolean | null;
  scope: string | null;
}

export interface ModelQuota {
  modelId: string;
  displayName: string;
  remainingPercent: number | null;
  resetAt: string | null;
  groupLabel: string | null;
  sourceExact: boolean;
}

export interface LocalTokens {
  schemaVersion: number;
  origin: string;
  rangeStart: string;
  rangeEnd: string;
  input: number | null;
  output: number | null;
  reasoning: number | null;
  cacheRead: number | null;
  cacheWrite: number | null;
  total: number | null;
  models: Array<{ modelId: string; tokens: number | null }>;
  completeness: string;
}

export interface ProviderDiag {
  providerId: ProviderId;
  status: Availability;
  origin: OriginKind;
  lastSuccess: string | null;
  lastAttempt: string | null;
  errorCode: string | null;
}

export interface SanitizedDiagnostics {
  timestamp: string;
  appVersion: string;
  os: string;
  providers: ProviderDiag[];
  opencodeDbDetected: boolean;
  antigravityAccountsDetected: boolean;
  codexCliDetected: boolean;
}

export interface PublicSettings {
  startWithWindows: boolean;
  theme: string;
  googleLiveOptIn: boolean;
  pollIntervalMins: number;
  notificationsEnabled: boolean;
  notifyThreshold25: boolean;
  notifyThreshold10: boolean;
  notifyThreshold0: boolean;
  pinned: boolean;
  showInTaskbar: boolean;
}

export interface SafeSettingsPatch {
  startWithWindows?: boolean;
  theme?: string;
  googleLiveOptIn?: boolean;
  pollIntervalMins?: number;
  notificationsEnabled?: boolean;
  notifyThreshold25?: boolean;
  notifyThreshold10?: boolean;
  notifyThreshold0?: boolean;
  pinned?: boolean;
  showInTaskbar?: boolean;
}
