import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { setToken, getToken } from "@/lib/token";
import { hydrate, type HydrateProgress } from "@/dict/hydrate";

export const Route = createFileRoute("/setup")({
  component: SetupPage,
});

function SetupPage() {
  const navigate = useNavigate();
  const [token, setTok] = useState(getToken() ?? "");
  const [progress, setProgress] = useState<HydrateProgress | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const start = async () => {
    setBusy(true);
    setError(null);
    if (token) setToken(token);
    try {
      await hydrate(setProgress);
      navigate({ to: "/" });
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setBusy(false);
    }
  };

  return (
    <div
      style={{
        maxWidth: 520,
        margin: "var(--s-7) auto",
        display: "grid",
        gap: "var(--s-4)",
      }}
    >
      <h1
        style={{
          fontFamily: "var(--font-serif, serif)",
          fontSize: "var(--t-display-lg)",
          margin: 0,
        }}
      >
        First-run setup
      </h1>
      <p style={{ color: "var(--ink-soft)", margin: 0 }}>
        Paste your NUKU_TOKEN, then we'll download the JMDict / WaniKani / frequency bundles into your browser.
      </p>
      <input
        type="password"
        autoFocus
        value={token}
        onChange={(e) => setTok(e.target.value)}
        placeholder="NUKU_TOKEN"
        disabled={busy}
        style={{
          padding: "var(--s-3) var(--s-4)",
          background: "var(--paper-alt)",
          color: "var(--ink)",
          border: "1px solid var(--rule-soft)",
          fontFamily: "var(--font-mono, monospace)",
        }}
      />
      <button className="btn btn-primary" disabled={!token || busy} onClick={start}>
        {busy ? "Working…" : "Continue"}
      </button>
      {progress && <Progress p={progress} />}
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
          {error}
        </pre>
      )}
    </div>
  );
}

function Progress({ p }: { p: HydrateProgress }) {
  const pct =
    p.total && p.loaded ? Math.floor((p.loaded / p.total) * 100) : null;
  return (
    <div style={{ color: "var(--ink-soft)" }}>
      {p.phase === "ready" ? "Done." : `${p.phase}…`}
      {pct !== null && ` ${pct}%`}
    </div>
  );
}
