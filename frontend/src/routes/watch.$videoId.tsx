import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useMemo, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/api/client";
import { SubtitleOverlay } from "@/player/SubtitleOverlay";
import { Controls } from "@/player/Controls";
import { Popup, type ResolvedEntry } from "@/popup/Popup";
import { MiningModal } from "@/mining/MiningModal";
import type { SubtitleLine, Token } from "@/api/types";

export const Route = createFileRoute("/watch/$videoId")({
  component: WatchPage,
});

function WatchPage() {
  const { videoId } = Route.useParams();
  const id = Number(videoId);
  const { data: video } = useQuery({
    queryKey: ["video", id],
    queryFn: () => api.video(id),
  });
  const { data: subtitles } = useQuery({
    queryKey: ["subtitles", id],
    queryFn: () => api.subtitles(id),
    staleTime: Infinity,
  });
  const { data: known } = useQuery({
    queryKey: ["known"],
    queryFn: api.knownWords,
    refetchInterval: 60_000,
  });
  const { data: progress } = useQuery({
    queryKey: ["progress", id],
    queryFn: () => api.progress(id),
    staleTime: 30_000,
  });

  const videoRef = useRef<HTMLVideoElement | null>(null);
  const [videoEl, setVideoEl] = useState<HTMLVideoElement | null>(null);
  const [currentMs, setCurrentMs] = useState(0);
  const [popupToken, setPopupToken] = useState<{ token: Token; rect: DOMRect } | null>(null);
  const [mining, setMining] = useState<{
    line: SubtitleLine;
    resolved: ResolvedEntry;
  } | null>(null);
  const [resumedFromProgress, setResumedFromProgress] = useState(false);

  const streamUrl = useMemo(() => api.streamUrl(id), [id]);

  useEffect(() => {
    if (!videoEl || !progress || resumedFromProgress) return;
    if (progress.position_ms > 5_000) {
      if (confirm(`Resume from ${formatTime(progress.position_ms)}? (Cancel = start over)`)) {
        videoEl.currentTime = progress.position_ms / 1000;
      }
    }
    setResumedFromProgress(true);
  }, [videoEl, progress, resumedFromProgress]);

  useEffect(() => {
    if (!videoEl) return;
    let raf = 0;
    const tick = () => {
      setCurrentMs(videoEl.currentTime * 1000);
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [videoEl]);

  useEffect(() => {
    if (!videoEl) return;
    let last = 0;
    const onTimeUpdate = () => {
      const now = Date.now();
      if (now - last < 5_000) return;
      last = now;
      const pos = Math.floor(videoEl.currentTime * 1000);
      void api.postProgress(id, {
        position_ms: pos,
        device: navigator.userAgent.slice(0, 80),
      });
    };
    videoEl.addEventListener("timeupdate", onTimeUpdate);
    return () => videoEl.removeEventListener("timeupdate", onTimeUpdate);
  }, [videoEl, id]);

  // Immersion session — start on mount, end on unmount/route change.
  useEffect(() => {
    if (!video) return;
    const startedAt = new Date().toISOString();
    const startedMs = Date.now();
    return () => {
      const endedAt = new Date().toISOString();
      const durationMs = Date.now() - startedMs;
      if (durationMs < 5_000) return; // ignore micro visits
      void api.postSession({
        kind: "video",
        source: "hibi-nuku",
        startedAt,
        endedAt,
        durationMs,
        metadata: { videoId: id, title: video.title },
      });
    };
  }, [video, id]);

  const activeLine = useMemo(() => {
    if (!subtitles) return null;
    return (
      subtitles.find((l) => currentMs >= l.startMs && currentMs <= l.endMs) ?? null
    );
  }, [subtitles, currentMs]);

  useShortcuts({ videoEl, subtitles: subtitles ?? [], current: activeLine, setPopup: setPopupToken });

  if (!video) return <p style={{ color: "var(--ink-soft)" }}>Loading…</p>;

  return (
    <div>
      <h1
        style={{
          fontFamily: "var(--font-serif, serif)",
          fontSize: "var(--t-display-md)",
          margin: 0,
          marginBottom: "var(--s-3)",
        }}
      >
        {video.title}
      </h1>
      <div
        style={{
          position: "relative",
          background: "#000",
          aspectRatio: "16/9",
        }}
      >
        <video
          ref={(el) => {
            videoRef.current = el;
            setVideoEl(el);
          }}
          src={streamUrl}
          style={{ width: "100%", height: "100%", display: "block" }}
          controls={false}
          playsInline
        />
        <SubtitleOverlay
          line={activeLine}
          known={known?.items ?? []}
          onTokenClick={(token, rect) => setPopupToken({ token, rect })}
        />
      </div>
      <Controls videoEl={videoEl} durationMs={video.durationMs} />
      {popupToken && (
        <Popup
          token={popupToken.token}
          anchor={popupToken.rect}
          videoTitle={video.title}
          toneTags={parseToneTags(activeLine?.toneTags)}
          onClose={() => setPopupToken(null)}
          onMine={(resolved) => {
            if (!activeLine) return;
            setMining({ line: activeLine, resolved });
            setPopupToken(null);
            videoEl?.pause();
          }}
        />
      )}
      {mining && (
        <MiningModal
          videoId={id}
          line={mining.line}
          focusWord={mining.resolved.lemma}
          focusWordReading={mining.resolved.reading}
          defaultEnglish={mining.line.translation ?? ""}
          defaultGrammarNote={mining.line.grammarNote ?? ""}
          defaultTags={[video.sourceTag]}
          onClose={() => setMining(null)}
        />
      )}
    </div>
  );
}

function useShortcuts({
  videoEl,
  subtitles,
  current,
  setPopup,
}: {
  videoEl: HTMLVideoElement | null;
  subtitles: SubtitleLine[];
  current: SubtitleLine | null;
  setPopup: (p: { token: Token; rect: DOMRect } | null) => void;
}) {
  useEffect(() => {
    if (!videoEl) return;
    const onKey = (e: KeyboardEvent) => {
      const tag = (e.target as HTMLElement | null)?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA") return;
      switch (e.key.toLowerCase()) {
        case " ":
          e.preventDefault();
          if (videoEl.paused) videoEl.play();
          else videoEl.pause();
          break;
        case "a":
          jumpToLineBy(videoEl, subtitles, -1);
          break;
        case "d":
          jumpToLineBy(videoEl, subtitles, 1);
          break;
        case "z":
          if (current) videoEl.currentTime = current.startMs / 1000;
          break;
        case "s":
          videoEl.pause();
          setPopup(null);
          // Mining shortcut hookup lands in Phase 15.
          break;
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [videoEl, subtitles, current, setPopup]);
}

function jumpToLineBy(
  videoEl: HTMLVideoElement,
  subtitles: SubtitleLine[],
  delta: number,
) {
  const currentMs = videoEl.currentTime * 1000;
  const idx = subtitles.findIndex((l) => l.startMs > currentMs);
  let targetIdx = idx === -1 ? subtitles.length - 1 : idx - 1;
  targetIdx = Math.max(0, Math.min(subtitles.length - 1, targetIdx + delta));
  const next = subtitles[targetIdx];
  if (next) videoEl.currentTime = next.startMs / 1000;
}

function parseToneTags(json: string | null | undefined): string[] {
  if (!json) return [];
  try {
    const v = JSON.parse(json);
    return Array.isArray(v) ? (v as string[]) : [];
  } catch {
    return [];
  }
}

function formatTime(ms: number): string {
  const total = Math.floor(ms / 1000);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  if (h) return `${h}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
  return `${m}:${s.toString().padStart(2, "0")}`;
}

