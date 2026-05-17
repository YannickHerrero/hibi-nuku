import { createFileRoute, Link } from "@tanstack/react-router";
import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "@/api/client";
import { ImportModal } from "@/library/ImportModal";
import type { Video, VideoStatus } from "@/api/types";

export const Route = createFileRoute("/")({
  component: LibraryIndex,
});

function LibraryIndex() {
  const [showImport, setShowImport] = useState(false);
  const { data, isLoading, error } = useQuery({
    queryKey: ["library"],
    queryFn: api.library,
    refetchInterval: (q) => {
      const list = q.state.data as Video[] | undefined;
      const pending = list?.some(
        (v) => v.status !== "ready" && v.status !== "error",
      );
      return pending ? 2000 : false;
    },
  });

  return (
    <div>
      <header
        style={{
          display: "flex",
          alignItems: "baseline",
          justifyContent: "space-between",
          marginBottom: "var(--s-5)",
        }}
      >
        <h1
          style={{
            fontFamily: "var(--font-serif, serif)",
            fontSize: "var(--t-display-lg)",
            letterSpacing: "var(--track-tight)",
            margin: 0,
          }}
        >
          Library
        </h1>
        <button className="btn btn-primary" onClick={() => setShowImport(true)}>
          Import video
        </button>
      </header>

      {error && (
        <pre
          style={{
            color: "#a33",
            background: "var(--paper-alt)",
            padding: "var(--s-3)",
            whiteSpace: "pre-wrap",
            fontSize: "var(--t-meta)",
          }}
        >
          {String(error)}
        </pre>
      )}

      {isLoading ? (
        <p style={{ color: "var(--ink-soft)" }}>Loading…</p>
      ) : !data || data.length === 0 ? (
        <Empty onImport={() => setShowImport(true)} />
      ) : (
        <Grid videos={data} />
      )}

      {showImport && <ImportModal onClose={() => setShowImport(false)} />}
    </div>
  );
}

function Empty({ onImport }: { onImport: () => void }) {
  return (
    <div
      style={{
        padding: "var(--s-7) var(--s-4)",
        textAlign: "center",
        color: "var(--ink-soft)",
      }}
    >
      <p>No videos imported yet.</p>
      <button className="btn btn-primary" onClick={onImport}>
        Import your first video
      </button>
    </div>
  );
}

function Grid({ videos }: { videos: Video[] }) {
  return (
    <ul
      style={{
        listStyle: "none",
        margin: 0,
        padding: 0,
        display: "grid",
        gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))",
        gap: "var(--s-4)",
      }}
    >
      {videos.map((v) => (
        <li key={v.id}>
          <Card v={v} />
        </li>
      ))}
    </ul>
  );
}

function Card({ v }: { v: Video }) {
  const ready = v.status === "ready";
  return (
    <div
      style={{
        background: "var(--paper-alt)",
        border: "1px solid var(--rule-soft)",
        display: "grid",
        gridTemplateRows: "auto 1fr auto",
      }}
    >
      <div
        style={{
          aspectRatio: "16/9",
          background: "var(--muted)",
          backgroundImage: `url(${api.thumbnailUrl(v.id)})`,
          backgroundSize: "cover",
          backgroundPosition: "center",
        }}
      />
      <div style={{ padding: "var(--s-3)" }}>
        <div
          style={{
            fontWeight: 600,
            fontSize: "var(--t-body)",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}
        >
          {v.title}
        </div>
        <div
          style={{
            color: "var(--ink-faint)",
            fontSize: "var(--t-meta)",
            fontFamily: "var(--font-mono, monospace)",
          }}
        >
          {v.sourceTag}
        </div>
      </div>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          padding: "var(--s-2) var(--s-3)",
          borderTop: "1px solid var(--rule-soft)",
        }}
      >
        <StatusBadge status={v.status} message={v.errorMessage} />
        {ready ? (
          <Link to="/watch/$videoId" params={{ videoId: String(v.id) }} className="btn">
            Watch
          </Link>
        ) : (
          <RowActions video={v} />
        )}
      </div>
    </div>
  );
}

function StatusBadge({
  status,
  message,
}: {
  status: VideoStatus;
  message: string | null;
}) {
  const color =
    status === "ready"
      ? "var(--accent)"
      : status === "error"
        ? "#a33"
        : "var(--ink-soft)";
  return (
    <span
      title={message ?? undefined}
      style={{
        color,
        fontFamily: "var(--font-mono, monospace)",
        fontSize: "var(--t-meta)",
        textTransform: "lowercase",
        letterSpacing: "var(--track-mono)",
      }}
    >
      {status}
    </span>
  );
}

function RowActions({ video }: { video: Video }) {
  const qc = useQueryClient();
  const reprocess = useMutation({
    mutationFn: () => api.reprocess(video.id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["library"] }),
  });
  const remove = useMutation({
    mutationFn: () => api.deleteVideo(video.id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["library"] }),
  });
  return (
    <div style={{ display: "flex", gap: "var(--s-2)" }}>
      {video.status === "error" && (
        <button className="btn btn-ghost" onClick={() => reprocess.mutate()}>
          Retry
        </button>
      )}
      <button
        className="btn btn-ghost"
        onClick={() => {
          if (confirm(`Remove "${video.title}" from the library?`)) {
            remove.mutate();
          }
        }}
      >
        Remove
      </button>
    </div>
  );
}

