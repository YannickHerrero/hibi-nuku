import { http } from "./http";
import type {
  Manifest,
  MineRequest,
  MineResponse,
  ProgressResp,
  Settings,
  SubtitleLine,
  Video,
} from "./types";
import type { KnownWord } from "./types";

export const api = {
  health: () => http.get<{ ok: true; version: string }>("/api/health"),

  // Library
  library: () => http.get<Video[]>("/api/library"),
  video: (id: number) => http.get<Video>(`/api/videos/${id}`),
  importVideo: (body: { path: string; title?: string; source_tag?: string }) =>
    http.post<{ videoId: number; status: string }>("/api/library/import", body),
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
  streamUrl: (id: number, fromSec?: number) =>
    `/api/videos/${id}/stream${fromSec ? `?from=${fromSec}` : ""}`,
  thumbnailUrl: (id: number) => `/api/videos/${id}/thumbnail`,

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
};
