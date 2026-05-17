import { useEffect, useRef, useState } from "react";

interface Props {
  videoEl: HTMLVideoElement | null;
  durationMs: number;
}

export function Controls({ videoEl, durationMs }: Props) {
  const [playing, setPlaying] = useState(false);
  const [currentMs, setCurrentMs] = useState(0);
  const rafRef = useRef<number | null>(null);

  useEffect(() => {
    if (!videoEl) return;
    const onPlay = () => setPlaying(true);
    const onPause = () => setPlaying(false);
    videoEl.addEventListener("play", onPlay);
    videoEl.addEventListener("pause", onPause);
    return () => {
      videoEl.removeEventListener("play", onPlay);
      videoEl.removeEventListener("pause", onPause);
    };
  }, [videoEl]);

  useEffect(() => {
    if (!videoEl) return;
    const tick = () => {
      setCurrentMs(videoEl.currentTime * 1000);
      rafRef.current = requestAnimationFrame(tick);
    };
    rafRef.current = requestAnimationFrame(tick);
    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
    };
  }, [videoEl]);

  if (!videoEl) return null;

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: "var(--s-3)",
        padding: "var(--s-3)",
        background: "var(--paper-alt)",
        borderTop: "1px solid var(--rule-soft)",
      }}
    >
      <button
        className="btn"
        onClick={() => (playing ? videoEl.pause() : videoEl.play())}
        title={playing ? "Pause" : "Play"}
      >
        {playing ? "❚❚" : "▶"}
      </button>
      <span
        style={{
          fontFamily: "var(--font-mono, monospace)",
          fontSize: "var(--t-meta)",
          minWidth: 110,
        }}
      >
        {formatTime(currentMs)} / {formatTime(durationMs)}
      </span>
      <input
        type="range"
        min={0}
        max={durationMs}
        value={currentMs}
        onChange={(e) => {
          const ms = Number(e.target.value);
          videoEl.currentTime = ms / 1000;
        }}
        style={{ flex: 1 }}
      />
    </div>
  );
}

function formatTime(ms: number): string {
  if (!isFinite(ms) || ms < 0) ms = 0;
  const total = Math.floor(ms / 1000);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  if (h) return `${h}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
  return `${m}:${s.toString().padStart(2, "0")}`;
}
