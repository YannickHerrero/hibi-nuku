import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "@/api/client";
import { getToken } from "@/lib/token";
import { getTheme } from "@/lib/theme";
import { openNukuDb, getMeta } from "@/dict/idb";
import { lookupSurface, frequencyRank, wkKanji } from "@/dict/lookup";
import { deinflect } from "@/dict/deinflect";

export const Route = createFileRoute("/debug")({
  component: DebugPage,
});

function DebugPage() {
  return (
    <div style={{ display: "grid", gap: "var(--s-6)" }}>
      <h1
        style={{
          fontFamily: "var(--font-serif, serif)",
          fontSize: "var(--t-display-lg)",
          margin: 0,
        }}
      >
        Debug
      </h1>

      <Client />
      <ServerSettings />
      <DbStats />
      <HibiPinger />
      <JmdictProbe />
      <IndexedDbInspector />
      <Caches />
      <LogsHint />
    </div>
  );
}

// ---------------- sections ----------------

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section style={{ display: "grid", gap: "var(--s-3)" }}>
      <h2
        style={{
          fontFamily: "var(--font-serif, serif)",
          fontSize: "var(--t-display-sm)",
          margin: 0,
          borderBottom: "1px solid var(--rule-soft)",
          paddingBottom: "var(--s-1)",
        }}
      >
        {title}
      </h2>
      {children}
    </section>
  );
}

function KV({ rows }: { rows: [string, React.ReactNode][] }) {
  return (
    <dl
      style={{
        display: "grid",
        gridTemplateColumns: "max-content 1fr",
        gap: "var(--s-2) var(--s-4)",
        margin: 0,
      }}
    >
      {rows.map(([k, v]) => (
        <div key={k} style={{ display: "contents" }}>
          <dt style={{ color: "var(--ink-soft)" }}>{k}</dt>
          <dd
            style={{
              margin: 0,
              fontFamily: "var(--font-mono, monospace)",
              fontSize: "var(--t-meta)",
              wordBreak: "break-all",
            }}
          >
            {v}
          </dd>
        </div>
      ))}
    </dl>
  );
}

function Client() {
  const token = getToken();
  const theme = getTheme();
  const ua = typeof navigator !== "undefined" ? navigator.userAgent : "—";
  const sw = useServiceWorkerStatus();
  return (
    <Section title="Client">
      <KV
        rows={[
          ["NUKU_TOKEN set", token ? "yes" : <Bad>no</Bad>],
          ["theme", theme],
          ["pathname", typeof location !== "undefined" ? location.pathname : "—"],
          ["user-agent", ua],
          ["service worker", sw],
        ]}
      />
    </Section>
  );
}

function ServerSettings() {
  const { data, error, refetch, isFetching } = useQuery({
    queryKey: ["debug-settings"],
    queryFn: api.settings,
    retry: false,
  });
  return (
    <Section title="Server settings">
      <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--s-2)" }}>
        <button className="btn" disabled={isFetching} onClick={() => refetch()}>
          {isFetching ? "…" : "Refresh"}
        </button>
      </div>
      {error && <Err>{String(error)}</Err>}
      {data && (
        <KV
          rows={[
            ["LLM model", data.llmModel],
            [
              "WK",
              data.wkUserLevel
                ? `level ${data.wkUserLevel} (${data.wkUsername ?? "?"})`
                : "—",
            ],
            ["Library dir", data.libraryDir],
            [
              "JMDict bundle",
              data.dict.jmdict
                ? `${data.dict.jmdict.version} ${formatBytes(data.dict.jmdict.size)} ${data.dict.jmdict.sha256.slice(0, 12)}…`
                : <Bad>missing</Bad>,
            ],
            [
              "WK bundle",
              data.dict.wk
                ? `${data.dict.wk.version} ${formatBytes(data.dict.wk.size)} ${data.dict.wk.sha256.slice(0, 12)}…`
                : <Bad>missing</Bad>,
            ],
            [
              "Frequency bundle",
              data.dict.frequency
                ? `${data.dict.frequency.version} ${formatBytes(data.dict.frequency.size)} ${data.dict.frequency.sha256.slice(0, 12)}…`
                : <Bad>missing</Bad>,
            ],
          ]}
        />
      )}
    </Section>
  );
}

function DbStats() {
  const { data, error, refetch, isFetching } = useQuery({
    queryKey: ["debug-db-stats"],
    queryFn: api.dbStats,
    retry: false,
  });
  return (
    <Section title="SQLite">
      <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--s-2)" }}>
        <button className="btn" disabled={isFetching} onClick={() => refetch()}>
          {isFetching ? "…" : "Refresh"}
        </button>
      </div>
      {error && <Err>{String(error)}</Err>}
      {data && (
        <>
          <KV
            rows={[
              ["path", data.db_path],
              ["file size", formatBytes(data.db_file_bytes)],
            ]}
          />
          <KV
            rows={Object.entries(data.tables).map(([t, n]) => [
              t,
              <span style={{ color: n === 0 ? "var(--ink-faint)" : undefined }}>{n}</span>,
            ])}
          />
        </>
      )}
    </Section>
  );
}

function HibiPinger() {
  const { data, error, refetch, isFetching } = useQuery({
    queryKey: ["debug-hibi-status"],
    queryFn: api.hibiStatus,
    enabled: false,
    retry: false,
  });
  return (
    <Section title="Hibi connectivity">
      <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--s-2)" }}>
        <button className="btn btn-primary" disabled={isFetching} onClick={() => refetch()}>
          {isFetching ? "Pinging…" : "Ping all"}
        </button>
      </div>
      {error && <Err>{String(error)}</Err>}
      {data && (
        <>
          <div style={{ color: "var(--ink-soft)", fontSize: "var(--t-meta)" }}>
            base: <code>{data.base}</code>
          </div>
          <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "var(--t-meta)" }}>
            <thead>
              <tr style={{ borderBottom: "1px solid var(--rule-soft)" }}>
                <Th>endpoint</Th>
                <Th>status</Th>
                <Th>latency</Th>
                <Th>body</Th>
              </tr>
            </thead>
            <tbody>
              {data.probes.map((p, i) => (
                <tr key={i} style={{ borderBottom: "1px solid var(--rule-soft)" }}>
                  <Td>
                    <code>
                      {p.method} {p.path}
                    </code>
                  </Td>
                  <Td>
                    <span style={{ color: p.ok ? "var(--accent)" : "#a33" }}>
                      {p.status ?? "—"}
                    </span>
                  </Td>
                  <Td>{p.latencyMs} ms</Td>
                  <Td style={{ maxWidth: 400 }}>
                    <code style={{ whiteSpace: "pre-wrap", wordBreak: "break-all" }}>
                      {p.bodySnippet ?? "—"}
                    </code>
                  </Td>
                </tr>
              ))}
            </tbody>
          </table>
        </>
      )}
    </Section>
  );
}

function JmdictProbe() {
  const [term, setTerm] = useState("食べる");
  const [results, setResults] = useState<unknown[]>([]);
  const [freq, setFreq] = useState<number | undefined>();
  const [cands, setCands] = useState<{ stem: string; rules: string[] }[]>([]);
  const [error, setError] = useState<string | null>(null);
  const run = async () => {
    setError(null);
    try {
      const r = await lookupSurface(term);
      const f = await frequencyRank(term);
      const c = deinflect(term).map((c) => ({ stem: c.stem, rules: [...c.rulesOut] }));
      setResults(r);
      setFreq(f);
      setCands(c);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };
  return (
    <Section title="JMDict probe (local IndexedDB)">
      <div style={{ display: "flex", gap: "var(--s-2)" }}>
        <input
          value={term}
          onChange={(e) => setTerm(e.target.value)}
          style={{
            flex: 1,
            padding: "var(--s-2) var(--s-3)",
            background: "var(--paper-alt)",
            border: "1px solid var(--rule-soft)",
            color: "var(--ink)",
            font: "inherit",
          }}
        />
        <button className="btn btn-primary" onClick={run}>
          Look up
        </button>
      </div>
      {error && <Err>{error}</Err>}
      {freq !== undefined && (
        <div style={{ color: "var(--ink-soft)" }}>
          JPDB rank: <code>#{freq.toLocaleString()}</code>
        </div>
      )}
      {cands.length > 0 && (
        <div>
          <div style={{ color: "var(--ink-soft)", fontSize: "var(--t-meta)" }}>
            Deinflection candidates
          </div>
          <ul
            style={{
              margin: 0,
              paddingLeft: "var(--s-4)",
              fontFamily: "var(--font-mono, monospace)",
              fontSize: "var(--t-meta)",
            }}
          >
            {cands.map((c, i) => (
              <li key={i}>
                <code>{c.stem}</code> {c.rules.length > 0 && `[${c.rules.join(",")}]`}
              </li>
            ))}
          </ul>
        </div>
      )}
      {results.length > 0 && (
        <details>
          <summary>{results.length} matching entries</summary>
          <pre
            style={{
              background: "var(--paper-alt)",
              padding: "var(--s-3)",
              fontSize: "var(--t-meta)",
              maxHeight: 400,
              overflow: "auto",
            }}
          >
            {JSON.stringify(results, null, 2)}
          </pre>
        </details>
      )}
    </Section>
  );
}

function IndexedDbInspector() {
  const { data, refetch, isFetching } = useQuery({
    queryKey: ["debug-idb"],
    queryFn: scanIndexedDb,
  });

  return (
    <Section title="IndexedDB (client)">
      <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--s-2)" }}>
        <button className="btn" disabled={isFetching} onClick={() => refetch()}>
          {isFetching ? "…" : "Refresh"}
        </button>
        <button
          className="btn"
          onClick={async () => {
            if (
              !confirm(
                "Wipe IndexedDB? You'll need to re-hydrate via /setup before the popup works.",
              )
            ) {
              return;
            }
            indexedDB.deleteDatabase("nuku-dict");
            location.reload();
          }}
        >
          Wipe + reload
        </button>
      </div>
      {data && (
        <KV
          rows={[
            ["jmdict_entries", data.counts.jmdict_entries],
            ["jmdict_index", data.counts.jmdict_index],
            ["wk_kanji", data.counts.wk_kanji],
            ["wk_vocab", data.counts.wk_vocab],
            ["wk_vocab_by_kanji", data.counts.wk_vocab_by_kanji],
            ["frequency", data.counts.frequency],
            ["meta: jmdict version", data.meta.jmdict ?? "—"],
            ["meta: wk version", data.meta.wk ?? "—"],
            ["meta: frequency version", data.meta.frequency ?? "—"],
            ["meta: wk_user_level", data.meta.wk_user_level ?? "—"],
          ]}
        />
      )}
    </Section>
  );
}

function Caches() {
  const qc = useQueryClient();
  const wipe = useMutation({
    mutationFn: (model: string | undefined) => api.wipeLlmCache(model),
  });
  const [model, setModel] = useState("");
  return (
    <Section title="Caches">
      <div style={{ display: "grid", gap: "var(--s-2)" }}>
        <div style={{ color: "var(--ink-soft)", fontSize: "var(--t-meta)" }}>
          Server-side LLM response cache (SQLite llm_cache table).
        </div>
        <div style={{ display: "flex", gap: "var(--s-2)" }}>
          <input
            value={model}
            onChange={(e) => setModel(e.target.value)}
            placeholder="model (blank = wipe all)"
            style={{
              flex: 1,
              padding: "var(--s-2) var(--s-3)",
              background: "var(--paper-alt)",
              border: "1px solid var(--rule-soft)",
              color: "var(--ink)",
              font: "inherit",
            }}
          />
          <button
            className="btn"
            onClick={() => wipe.mutate(model.trim() || undefined)}
            disabled={wipe.isPending}
          >
            {wipe.isPending ? "Wiping…" : "Wipe llm_cache"}
          </button>
        </div>
        {wipe.data && (
          <div style={{ color: "var(--accent)" }}>
            Deleted {wipe.data.deleted} row(s).
          </div>
        )}
        {wipe.error && <Err>{String(wipe.error)}</Err>}
      </div>
      <hr />
      <div style={{ display: "grid", gap: "var(--s-2)" }}>
        <div style={{ color: "var(--ink-soft)", fontSize: "var(--t-meta)" }}>
          TanStack Query in-memory cache.
        </div>
        <div>
          <button
            className="btn"
            onClick={() => {
              qc.clear();
              alert("Query cache cleared.");
            }}
          >
            Clear query cache
          </button>
        </div>
      </div>
    </Section>
  );
}

function LogsHint() {
  return (
    <Section title="Server logs">
      <p style={{ color: "var(--ink-soft)", margin: 0 }}>
        Backend logs are on stderr of the <code>nuku</code> process. For more verbosity:
      </p>
      <pre
        style={{
          background: "var(--paper-alt)",
          padding: "var(--s-3)",
          fontSize: "var(--t-meta)",
          margin: 0,
        }}
      >
{`NUKU_LOG="hibi_nuku=debug,tower_http=debug" cargo run --release --bin nuku`}
      </pre>
    </Section>
  );
}

// ---------------- helpers ----------------

function Th(props: React.ThHTMLAttributes<HTMLTableCellElement>) {
  return (
    <th
      {...props}
      style={{
        textAlign: "left",
        padding: "var(--s-2) var(--s-3)",
        color: "var(--ink-soft)",
        fontWeight: 500,
        ...props.style,
      }}
    />
  );
}

function Td(props: React.TdHTMLAttributes<HTMLTableCellElement>) {
  return (
    <td
      {...props}
      style={{
        padding: "var(--s-2) var(--s-3)",
        verticalAlign: "top",
        ...props.style,
      }}
    />
  );
}

function Bad({ children }: { children: React.ReactNode }) {
  return <span style={{ color: "#a33" }}>{children}</span>;
}

function Err({ children }: { children: React.ReactNode }) {
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
      {children}
    </pre>
  );
}

function formatBytes(b: number): string {
  if (b > 1024 * 1024 * 1024) return `${(b / 1024 / 1024 / 1024).toFixed(2)} GB`;
  if (b > 1024 * 1024) return `${(b / 1024 / 1024).toFixed(1)} MB`;
  if (b > 1024) return `${(b / 1024).toFixed(1)} KB`;
  return `${b} B`;
}

function useServiceWorkerStatus() {
  const [status, setStatus] = useState<string>("…");
  useEffect(() => {
    if (typeof navigator === "undefined" || !("serviceWorker" in navigator)) {
      setStatus("unavailable");
      return;
    }
    navigator.serviceWorker
      .getRegistrations()
      .then((regs) => {
        if (regs.length === 0) setStatus("none registered");
        else
          setStatus(
            regs
              .map((r) => `${r.scope} (${r.active?.state ?? "inactive"})`)
              .join(", "),
          );
      })
      .catch((e) => setStatus(String(e)));
  }, []);
  return status;
}

async function scanIndexedDb() {
  const db = await openNukuDb();
  const stores = [
    "jmdict_entries",
    "jmdict_index",
    "wk_kanji",
    "wk_vocab",
    "wk_vocab_by_kanji",
    "frequency",
  ] as const;
  const counts: Record<string, number> = {};
  for (const s of stores) {
    counts[s] = await db.count(s);
  }
  const meta = {
    jmdict: await getMeta("jmdict"),
    wk: await getMeta("wk"),
    frequency: await getMeta("frequency"),
    wk_user_level: await getMeta("wk_user_level"),
  };
  // Suppress unused-imports warnings (kanji helper used by the popup elsewhere)
  void wkKanji;
  return { counts: counts as Record<string, number>, meta };
}
