import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { THEMES, getTheme, setTheme, type Theme } from "@/lib/theme";
import { getToken, setToken, clearToken } from "@/lib/token";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/api/client";

export const Route = createFileRoute("/settings")({
  component: SettingsPage,
});

function SettingsPage() {
  const [theme, set] = useState<Theme>(getTheme());
  const [token, setTok] = useState(getToken() ?? "");
  const { data: settings, isLoading } = useQuery({
    queryKey: ["settings"],
    queryFn: api.settings,
    retry: false,
  });

  useEffect(() => {
    setTheme(theme);
  }, [theme]);

  return (
    <div style={{ display: "grid", gap: "var(--s-6)" }}>
      <Section title="Theme">
        <div style={{ display: "flex", gap: "var(--s-2)", flexWrap: "wrap" }}>
          {THEMES.map((t) => (
            <button
              key={t}
              className={`btn ${t === theme ? "btn-primary" : ""}`}
              onClick={() => set(t)}
              style={{ textTransform: "lowercase" }}
            >
              {t}
            </button>
          ))}
        </div>
      </Section>

      <Section title="Server token">
        <input
          type="password"
          value={token}
          onChange={(e) => setTok(e.target.value)}
          placeholder="NUKU_TOKEN"
          style={{
            flex: 1,
            padding: "var(--s-2) var(--s-3)",
            background: "var(--paper-alt)",
            color: "var(--ink)",
            border: "1px solid var(--rule-soft)",
          }}
        />
        <button
          className="btn btn-primary"
          onClick={() => {
            setToken(token);
            window.location.reload();
          }}
        >
          Save & reload
        </button>
        <button className="btn" onClick={() => {
          clearToken();
          setTok("");
        }}>
          Clear
        </button>
      </Section>

      <Section title="Server info">
        {isLoading ? (
          <p style={{ color: "var(--ink-soft)" }}>Loading…</p>
        ) : settings ? (
          <dl style={{ display: "grid", gridTemplateColumns: "auto 1fr", gap: "var(--s-2) var(--s-4)" }}>
            <dt style={{ color: "var(--ink-soft)" }}>LLM model</dt>
            <dd>{settings.llmModel}</dd>
            <dt style={{ color: "var(--ink-soft)" }}>WK level</dt>
            <dd>{settings.wkUserLevel ?? "—"} {settings.wkUsername ? `(${settings.wkUsername})` : ""}</dd>
            <dt style={{ color: "var(--ink-soft)" }}>Library dir</dt>
            <dd style={{ fontFamily: "var(--font-mono, monospace)" }}>{settings.libraryDir}</dd>
            <dt style={{ color: "var(--ink-soft)" }}>JMDict bundle</dt>
            <dd>{settings.dict.jmdict ? `${settings.dict.jmdict.version} (${formatBytes(settings.dict.jmdict.size)})` : "missing"}</dd>
            <dt style={{ color: "var(--ink-soft)" }}>WK bundle</dt>
            <dd>{settings.dict.wk ? `${settings.dict.wk.version} (${formatBytes(settings.dict.wk.size)})` : "missing"}</dd>
            <dt style={{ color: "var(--ink-soft)" }}>Frequency bundle</dt>
            <dd>{settings.dict.frequency ? `${settings.dict.frequency.version} (${formatBytes(settings.dict.frequency.size)})` : "missing"}</dd>
          </dl>
        ) : (
          <p style={{ color: "var(--ink-soft)" }}>Server unreachable or token missing.</p>
        )}
      </Section>
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section>
      <h2
        style={{
          fontFamily: "var(--font-serif, serif)",
          fontSize: "var(--t-display-sm)",
          margin: 0,
          marginBottom: "var(--s-3)",
        }}
      >
        {title}
      </h2>
      <div style={{ display: "flex", gap: "var(--s-2)", flexWrap: "wrap", alignItems: "center" }}>
        {children}
      </div>
    </section>
  );
}

function formatBytes(b: number): string {
  if (b > 1024 * 1024) return `${(b / 1024 / 1024).toFixed(1)} MB`;
  if (b > 1024) return `${(b / 1024).toFixed(1)} KB`;
  return `${b} B`;
}
