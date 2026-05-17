import { useEffect, useRef, useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/api/client";
import { uploadMedia, uploadSubtitle } from "@/api/upload";
import type { ProbeTrack, UploadMediaResp } from "@/api/types";

interface Props {
  onClose: () => void;
}

type Step =
  | { kind: "pick-video" }
  | { kind: "uploading-video"; loaded: number; total: number | null; name: string }
  | { kind: "configure"; upload: UploadMediaResp; name: string }
  | { kind: "submitting" };

export function ImportModal({ onClose }: Props) {
  const [step, setStep] = useState<Step>({ kind: "pick-video" });
  const [error, setError] = useState<string | null>(null);
  const abortRef = useRef<AbortController | null>(null);

  const cancel = () => {
    abortRef.current?.abort();
    onClose();
  };

  const onPick = async (file: File) => {
    setError(null);
    setStep({ kind: "uploading-video", loaded: 0, total: file.size, name: file.name });
    const ctrl = new AbortController();
    abortRef.current = ctrl;
    try {
      const resp = await uploadMedia(file, {
        signal: ctrl.signal,
        onProgress: (loaded, total) =>
          setStep({ kind: "uploading-video", loaded, total, name: file.name }),
      });
      setStep({ kind: "configure", upload: resp, name: file.name });
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setStep({ kind: "pick-video" });
    }
  };

  return (
    <Backdrop onClose={cancel}>
      <header style={{ display: "flex", alignItems: "baseline", justifyContent: "space-between" }}>
        <h2 style={{ margin: 0, fontFamily: "var(--font-serif, serif)" }}>Import video</h2>
        <button className="btn btn-ghost" onClick={cancel}>
          ✕
        </button>
      </header>

      {error && <ErrorBox text={error} />}

      {step.kind === "pick-video" && <PickVideo onPick={onPick} />}
      {step.kind === "uploading-video" && (
        <UploadingProgress loaded={step.loaded} total={step.total} name={step.name} onCancel={cancel} />
      )}
      {step.kind === "configure" && (
        <Configure
          upload={step.upload}
          name={step.name}
          onError={setError}
          onSubmittingChange={(s) => setStep(s ? { kind: "submitting" } : { kind: "configure", upload: step.upload, name: step.name })}
          onClose={onClose}
        />
      )}
      {step.kind === "submitting" && (
        <p style={{ color: "var(--ink-soft)", margin: 0 }}>Starting import…</p>
      )}
    </Backdrop>
  );
}

// ---------------- step views ----------------

function PickVideo({ onPick }: { onPick: (file: File) => void }) {
  const inputRef = useRef<HTMLInputElement>(null);
  return (
    <div style={{ display: "grid", gap: "var(--s-3)" }}>
      <p style={{ margin: 0, color: "var(--ink-soft)" }}>
        Pick a video file (mkv, mp4, …). It uploads to the server then we read its tracks.
      </p>
      <input
        ref={inputRef}
        type="file"
        accept="video/*,.mkv,.mp4,.webm,.avi"
        style={{ display: "none" }}
        onChange={(e) => {
          const f = e.target.files?.[0];
          if (f) onPick(f);
        }}
      />
      <button className="btn btn-primary" onClick={() => inputRef.current?.click()}>
        Choose video
      </button>
    </div>
  );
}

function UploadingProgress({
  loaded,
  total,
  name,
  onCancel,
}: {
  loaded: number;
  total: number | null;
  name: string;
  onCancel: () => void;
}) {
  const pct = total ? Math.floor((loaded / total) * 100) : null;
  const { bytesPerSec, etaSec } = useTransferStats(loaded, total);
  return (
    <div style={{ display: "grid", gap: "var(--s-3)" }}>
      <p style={{ margin: 0, color: "var(--ink-soft)" }}>Uploading {name}…</p>
      <div style={{ background: "var(--paper-alt)", height: 8, overflow: "hidden" }}>
        <div
          style={{
            background: "var(--accent)",
            height: "100%",
            width: `${pct ?? 5}%`,
            transition: "width 100ms linear",
          }}
        />
      </div>
      <div
        style={{
          color: "var(--ink-soft)",
          fontSize: "var(--t-meta)",
          display: "flex",
          justifyContent: "space-between",
          gap: "var(--s-3)",
          flexWrap: "wrap",
        }}
      >
        <span>
          {formatBytes(loaded)}
          {total ? ` / ${formatBytes(total)}` : ""} ({pct ?? "…"}%)
        </span>
        <span>
          {bytesPerSec !== null ? `${formatBytes(bytesPerSec)}/s` : "—"}
          {etaSec !== null ? ` · ${formatDuration(etaSec)} left` : ""}
        </span>
      </div>
      <div style={{ display: "flex", justifyContent: "flex-end" }}>
        <button className="btn btn-ghost" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </div>
  );
}

interface ConfigureProps {
  upload: UploadMediaResp;
  name: string;
  onError: (e: string | null) => void;
  onSubmittingChange: (submitting: boolean) => void;
  onClose: () => void;
}

function Configure({ upload, name, onError, onSubmittingChange, onClose }: ConfigureProps) {
  const qc = useQueryClient();
  const subtitleTracks = upload.probe.subtitle;
  const audioTracks = upload.probe.audio;

  const initialAudio =
    audioTracks.find((t) => isJa(t))?.index ?? audioTracks[0]?.index ?? null;
  // Subtitle: -1 means "upload sidecar"; null means "no subs at all" (rare).
  const initialSub: number | "sidecar" | null = (() => {
    const usable = subtitleTracks.find((t) => isJa(t) && !t.isImage);
    if (usable) return usable.index;
    if (subtitleTracks.length > 0 && subtitleTracks.every((t) => t.isImage)) return "sidecar";
    return subtitleTracks[0]?.index ?? "sidecar";
  })();

  const [audioIdx, setAudioIdx] = useState<number | null>(initialAudio);
  const [subChoice, setSubChoice] = useState<number | "sidecar" | null>(initialSub);
  const [sidecarPath, setSidecarPath] = useState<string | null>(null);
  const [sidecarFormat, setSidecarFormat] = useState<string | null>(null);
  const [sidecarUploading, setSidecarUploading] = useState(false);
  const sidecarInputRef = useRef<HTMLInputElement>(null);

  const baseName = stripExt(name);
  const [title, setTitle] = useState(baseName);
  const [sourceTag, setSourceTag] = useState(baseName);

  const submit = useMutation({
    mutationFn: async () => {
      onError(null);
      onSubmittingChange(true);
      const selectedSub =
        subChoice === "sidecar"
          ? null
          : subtitleTracks.find((t) => t.index === subChoice) ?? null;

      const subFormat = subChoice === "sidecar"
        ? sidecarFormat
        : selectedSub?.codecName ?? null;

      return api.createFromUpload({
        path: upload.path,
        title,
        sourceTag,
        jpAudioIdx: audioIdx,
        jpSubtitleIdx: subChoice === "sidecar" ? null : subChoice,
        subtitleFormat: subFormat,
        subtitleSidecarPath: subChoice === "sidecar" ? sidecarPath : null,
      });
    },
    onSettled: () => onSubmittingChange(false),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["library"] });
      onClose();
    },
    onError: (e) => onError(e instanceof Error ? e.message : String(e)),
  });

  const needSidecar = subChoice === "sidecar";
  const canSubmit =
    audioIdx !== null && !submit.isPending && !sidecarUploading && (!needSidecar || sidecarPath !== null);

  const pickSidecar = async (f: File) => {
    setSidecarUploading(true);
    onError(null);
    try {
      const resp = await uploadSubtitle(f);
      setSidecarPath(resp.path);
      setSidecarFormat(resp.format);
    } catch (e) {
      onError(e instanceof Error ? e.message : String(e));
    } finally {
      setSidecarUploading(false);
    }
  };

  return (
    <div style={{ display: "grid", gap: "var(--s-3)" }}>
      <Field label="Title">
        <input value={title} onChange={(e) => setTitle(e.target.value)} style={inputStyle} />
      </Field>
      <Field label="Source tag (used as Hibi card `source`)">
        <input value={sourceTag} onChange={(e) => setSourceTag(e.target.value)} style={inputStyle} />
      </Field>

      <Field label={`Audio track (${audioTracks.length})`}>
        <select
          value={audioIdx ?? ""}
          onChange={(e) => setAudioIdx(Number(e.target.value))}
          style={inputStyle}
        >
          {audioTracks.map((t) => (
            <option key={t.index} value={t.index}>
              {trackLabel(t)}
            </option>
          ))}
          {audioTracks.length === 0 && <option value="">no audio tracks</option>}
        </select>
      </Field>

      <Field label={`Subtitle (${subtitleTracks.length} embedded)`}>
        <select
          value={subChoice ?? ""}
          onChange={(e) =>
            setSubChoice(e.target.value === "sidecar" ? "sidecar" : Number(e.target.value))
          }
          style={inputStyle}
        >
          {subtitleTracks.map((t) => (
            <option key={t.index} value={t.index} disabled={t.isImage}>
              {trackLabel(t)}
              {t.isImage ? " — image-based (unsupported)" : ""}
            </option>
          ))}
          <option value="sidecar">Upload a sidecar file (.srt / .ass / .vtt)</option>
        </select>
      </Field>

      {needSidecar && (
        <div style={{ display: "grid", gap: "var(--s-2)" }}>
          <input
            ref={sidecarInputRef}
            type="file"
            accept=".srt,.ass,.ssa,.vtt"
            style={{ display: "none" }}
            onChange={(e) => {
              const f = e.target.files?.[0];
              if (f) pickSidecar(f);
            }}
          />
          {sidecarPath ? (
            <div style={{ color: "var(--ink-soft)", fontSize: "var(--t-meta)" }}>
              Sidecar uploaded: <code>{sidecarPath.split("/").pop()}</code> ({sidecarFormat})
            </div>
          ) : (
            <button
              className="btn"
              disabled={sidecarUploading}
              onClick={() => sidecarInputRef.current?.click()}
            >
              {sidecarUploading ? "Uploading…" : "Choose sidecar file"}
            </button>
          )}
        </div>
      )}

      <footer style={{ display: "flex", justifyContent: "flex-end", gap: "var(--s-2)" }}>
        <button className="btn" onClick={onClose} disabled={submit.isPending}>
          Cancel
        </button>
        <button
          className="btn btn-primary"
          disabled={!canSubmit}
          onClick={() => submit.mutate()}
        >
          Start processing
        </button>
      </footer>
    </div>
  );
}

// ---------------- bits ----------------

function isJa(t: ProbeTrack): boolean {
  return t.language === "jpn" || t.language === "ja";
}

function trackLabel(t: ProbeTrack): string {
  const parts = [`#${t.index}`, t.codecName];
  if (t.language) parts.push(t.language);
  if (t.title) parts.push(`"${t.title}"`);
  return parts.join(" · ");
}

function stripExt(name: string): string {
  const i = name.lastIndexOf(".");
  return i > 0 ? name.slice(0, i) : name;
}

// Smoothed transfer rate + ETA from progress samples. Returns null
// values until we have enough samples to compute a stable rate.
function useTransferStats(
  loaded: number,
  total: number | null,
): { bytesPerSec: number | null; etaSec: number | null } {
  const startRef = useRef<{ t: number; loaded: number } | null>(null);
  const lastRef = useRef<{ t: number; loaded: number } | null>(null);
  const [bytesPerSec, setBytesPerSec] = useState<number | null>(null);

  useEffect(() => {
    const now = performance.now();
    if (startRef.current === null) {
      startRef.current = { t: now, loaded };
      lastRef.current = { t: now, loaded };
      return;
    }
    const last = lastRef.current!;
    const dt = (now - last.t) / 1000;
    if (dt < 0.25) return; // throttle: at most ~4 samples/sec
    const instant = (loaded - last.loaded) / dt;
    // Exponential moving average to smooth out browser jitter.
    setBytesPerSec((prev) => (prev === null ? instant : prev * 0.6 + instant * 0.4));
    lastRef.current = { t: now, loaded };
  }, [loaded]);

  const etaSec =
    total !== null && bytesPerSec !== null && bytesPerSec > 1024
      ? Math.max(0, (total - loaded) / bytesPerSec)
      : null;

  return { bytesPerSec, etaSec };
}

function formatDuration(sec: number): string {
  if (!isFinite(sec)) return "—";
  const s = Math.round(sec);
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  const rem = s % 60;
  if (m < 60) return rem === 0 ? `${m}m` : `${m}m ${rem}s`;
  const h = Math.floor(m / 60);
  const mRem = m % 60;
  return mRem === 0 ? `${h}h` : `${h}h ${mRem}m`;
}

function formatBytes(b: number): string {
  if (b > 1024 * 1024 * 1024) return `${(b / 1024 / 1024 / 1024).toFixed(2)} GB`;
  if (b > 1024 * 1024) return `${(b / 1024 / 1024).toFixed(1)} MB`;
  if (b > 1024) return `${(b / 1024).toFixed(1)} KB`;
  return `${b} B`;
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label style={{ display: "grid", gap: "var(--s-1)" }}>
      <span style={{ color: "var(--ink-soft)", fontSize: "var(--t-meta)" }}>{label}</span>
      {children}
    </label>
  );
}

function Backdrop({ onClose, children }: { onClose: () => void; children: React.ReactNode }) {
  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(0,0,0,0.45)",
        display: "grid",
        placeItems: "center",
        zIndex: 50,
      }}
      onClick={onClose}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          background: "var(--paper)",
          border: "1px solid var(--ink)",
          padding: "var(--s-5)",
          minWidth: 520,
          maxWidth: "92vw",
          maxHeight: "92vh",
          overflow: "auto",
          display: "grid",
          gap: "var(--s-3)",
        }}
      >
        {children}
      </div>
    </div>
  );
}

function ErrorBox({ text }: { text: string }) {
  return (
    <pre
      style={{
        color: "#a33",
        background: "var(--paper-alt)",
        padding: "var(--s-3)",
        whiteSpace: "pre-wrap",
        fontSize: "var(--t-meta)",
        margin: 0,
      }}
    >
      {text}
    </pre>
  );
}

const inputStyle: React.CSSProperties = {
  padding: "var(--s-2) var(--s-3)",
  background: "var(--paper-alt)",
  color: "var(--ink)",
  border: "1px solid var(--rule-soft)",
  font: "inherit",
  width: "100%",
  boxSizing: "border-box",
};
