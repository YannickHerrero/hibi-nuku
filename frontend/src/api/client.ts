import { http } from "./http";
import { getToken } from "@/lib/token";
import type {
  CreateReq,
  CreateResp,
  Manifest,
  MineRequest,
  MineResponse,
  ProgressResp,
  Settings,
  SubtitleLine,
  Video,
} from "./types";
import type { KnownWord } from "./types";

// `<img src>` / `<video src>` can't carry custom headers, so for
// protected media we append the bearer as a query param. The backend
// auth middleware accepts either form.
function withToken(path: string): string {
  const t = getToken();
  if (!t) return path;
  const sep = path.includes("?") ? "&" : "?";
  return `${path}${sep}token=${encodeURIComponent(t)}`;
}

export const api = {
  health: () => http.get<{ ok: true; version: string }>("/api/health"),

  // Library
  library: () => http.get<Video[]>("/api/library"),
  video: (id: number) => http.get<Video>(`/api/videos/${id}`),
  createFromUpload: (body: CreateReq) =>
    http.post<CreateResp>("/api/library/create", body),
  patchVideo: (id: number, body: { title?: string; source_tag?: string }) =>
    http.patch<Video>(`/api/videos/${id}`, body),
  deleteVideo: (id: number) =>
    http.delete<{ deleted: boolean }>(`/api/videos/${id}`),
  reprocess: (id: number) =>
    http.post<{ videoId: number; status: string }>(`/api/videos/${id}/reprocess`),

  // Player / mining
  subtitles: (id: number) => http.get<SubtitleLine[]>(`/api/videos/${id}/subtitles`),
  progress: (id: number) => http.get<ProgressResp>(`/api/videos/${id}/progress`),
  postProgress: (id: number, body: { position_ms: number; device?: string }) =>
    http.post<ProgressResp>(`/api/videos/${id}/progress`, body),
  // Stream URL — browser handles seek natively via HTTP Range when
  // the backend has a pre-remuxed file. The legacy `?from=<sec>`
  // reload-on-seek param is still accepted as a fallback for videos
  // imported before the remux pipeline step shipped.
  streamUrl: (id: number, fromSec?: number) =>
    withToken(`/api/videos/${id}/stream${fromSec ? `?from=${fromSec}` : ""}`),
  thumbnailUrl: (id: number) => withToken(`/api/videos/${id}/thumbnail`),

  // Mining + Hibi
  mine: (body: MineRequest) => http.post<MineResponse>("/api/mine", body),
  knownWords: () => http.get<{ items: KnownWord[] }>("/api/known-words"),
  putWordStatus: (body: {
    lemma: string;
    reading: string;
    status: "learning" | "known" | "ignored" | null;
  }) => http.put<unknown>("/api/word-status", body),
  postSession: (body: {
    kind: string;
    source: string;
    startedAt: string;
    endedAt: string;
    durationMs: number;
    metadata?: Record<string, unknown>;
  }) => http.post<unknown>("/api/sessions", body),

  // Dict + settings
  dictManifest: () => http.get<Manifest>("/api/dict/manifest"),
  settings: () => http.get<Settings>("/api/settings"),

  // Debug
  hibiStatus: () =>
    http.get<{
      base: string;
      probes: {
        method: string;
        path: string;
        status: number | null;
        latencyMs: number;
        bodySnippet: string | null;
        ok: boolean;
      }[];
    }>("/api/debug/hibi-status"),
  dbStats: () =>
    http.get<{
      db_path: string;
      db_file_bytes: number;
      tables: Record<string, number>;
    }>("/api/debug/db-stats"),
  wipeLlmCache: (model?: string) =>
    http.delete<{ deleted: number }>(
      `/api/debug/llm-cache${model ? `?model=${encodeURIComponent(model)}` : ""}`,
    ),
};
