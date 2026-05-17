// Wire shapes shared between backend and frontend.

export type VideoStatus =
  | "probing"
  | "extracting"
  | "parsing"
  | "tokenizing"
  | "translating"
  | "ready"
  | "error";

export interface Video {
  id: number;
  path: string;
  title: string;
  sourceTag: string;
  durationMs: number;
  jpAudioIdx: number | null;
  jpSubtitleIdx: number | null;
  subtitleFormat: string | null;
  status: VideoStatus;
  errorMessage: string | null;
  importedAt: string;
  updatedAt: string;
}

export interface SubtitleLine {
  id: number;
  videoId: number;
  idx: number;
  startMs: number;
  endMs: number;
  rawText: string;
  tokensJson: string | null;
  translation: string | null;
  grammarNote: string | null;
  toneTags: string | null;
}

export interface Token {
  span: [number, number];
  surface: string;
  lemma: string;
  reading: string;
  pos: string;
  dict_seq: number | null;
}

export interface ProgressResp {
  position_ms: number;
  last_watched_at: string | null;
  last_device: string | null;
}

export interface DictBundleMeta {
  url: string;
  size: number;
  sha256: string;
  version: string;
}

export interface Manifest {
  jmdict: DictBundleMeta | null;
  wk: DictBundleMeta | null;
  frequency: DictBundleMeta | null;
}

export interface Settings {
  llmModel: string;
  wkUserLevel: number | null;
  wkUsername: string | null;
  libraryDir: string;
  dict: Manifest;
}

export interface KnownWord {
  lemma: string;
  reading: string;
  status: "learning" | "known" | "ignored";
}

export interface MineRequest {
  videoId: number;
  lineId: number;
  focusWord: string;
  focusWordReading: string;
  padBeforeMs?: number;
  padAfterMs?: number;
  userEnglishOverride?: string | null;
  userGrammarOverride?: string | null;
  tags?: string[];
}

export interface MineResponse {
  cardId: string;
  audioKey: string;
  imageKey: string;
}
