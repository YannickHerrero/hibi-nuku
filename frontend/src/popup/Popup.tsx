import { useEffect, useMemo, useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "@/api/client";
import type { Token } from "@/api/types";
import { lookupSurface, wkKanji, wkVocabByKanji, frequencyRank } from "@/dict/lookup";
import { deinflect } from "@/dict/deinflect";
import type { JmEntry, WkKanji } from "@/dict/idb";

interface Props {
  token: Token;
  anchor: DOMRect;
  videoTitle?: string;
  toneTags?: string[];
  onClose: () => void;
  onMine?: (entry: ResolvedEntry) => void;
}

export interface ResolvedEntry {
  surface: string;
  lemma: string;
  reading: string;
  glosses: string[];
  jm: JmEntry | null;
}

export function Popup({ token, anchor, toneTags, onClose, onMine }: Props) {
  const qc = useQueryClient();
  const ref = useRef<HTMLDivElement>(null);
  const pos = positionAnchored(anchor);

  // Resolve the token to JMDict entries (start from lemma, fall back
  // to deinflection paths).
  const { data: entries } = useQuery({
    queryKey: ["jm-resolve", token.lemma, token.surface],
    queryFn: async () => {
      const seqs: JmEntry[] = [];
      const tried = new Set<string>();
      const tryLookup = async (s: string) => {
        if (tried.has(s)) return;
        tried.add(s);
        const found = await lookupSurface(s);
        for (const e of found) if (!seqs.find((x) => x.seq === e.seq)) seqs.push(e);
      };
      await tryLookup(token.lemma);
      await tryLookup(token.surface);
      for (const cand of deinflect(token.surface)) await tryLookup(cand.stem);
      return seqs;
    },
  });
  const primary = entries?.[0] ?? null;
  const reading = primary?.readings?.[0] ?? token.reading;
  const lemma = primary?.kanji?.[0] ?? token.lemma;

  const { data: freq } = useQuery({
    queryKey: ["freq", lemma],
    queryFn: () => frequencyRank(lemma),
  });

  const { data: known } = useQuery({
    queryKey: ["known"],
    queryFn: api.knownWords,
    refetchInterval: 60_000,
    retry: false,
  });
  const knownStatus = useMemo(() => {
    if (!known) return undefined;
    return known.items.find((w) => w.lemma === lemma)?.status;
  }, [known, lemma]);

  const setStatus = useMutation({
    mutationFn: async (status: "learning" | "known" | "ignored" | null) => {
      await api.putWordStatus({ lemma, reading, status });
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ["known"] }),
  });

  // Close on outside click + Escape.
  useEffect(() => {
    const onDown = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    };
    const onEsc = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("mousedown", onDown);
    document.addEventListener("keydown", onEsc);
    return () => {
      document.removeEventListener("mousedown", onDown);
      document.removeEventListener("keydown", onEsc);
    };
  }, [onClose]);

  const glosses = primary?.senses.flatMap((s) => s.glosses).slice(0, 3) ?? [];

  return (
    <div
      ref={ref}
      style={{
        position: "fixed",
        top: pos.top,
        left: pos.left,
        background: "var(--paper)",
        border: "1px solid var(--ink)",
        boxShadow: "0 4px 18px rgba(0,0,0,0.18)",
        minWidth: 320,
        maxWidth: 420,
        zIndex: 50,
        padding: "var(--s-4)",
        display: "grid",
        gap: "var(--s-3)",
      }}
    >
      <header style={{ display: "flex", alignItems: "baseline", gap: "var(--s-3)" }}>
        <div style={{ fontFamily: "var(--font-serif, serif)", fontSize: "var(--t-display-md)" }}>
          {lemma}
        </div>
        <div style={{ color: "var(--ink-soft)" }}>{reading}</div>
        <span
          onClick={() => {
            const next: ("learning" | "known" | "ignored" | null) =
              knownStatus === undefined
                ? "learning"
                : knownStatus === "learning"
                  ? "known"
                  : knownStatus === "known"
                    ? "ignored"
                    : null;
            setStatus.mutate(next);
          }}
          title="Click to cycle: new → learning → known → ignored"
          style={{
            marginLeft: "auto",
            cursor: "pointer",
            fontFamily: "var(--font-mono, monospace)",
            fontSize: "var(--t-meta)",
            letterSpacing: "var(--track-mono)",
            color:
              knownStatus === "known"
                ? "var(--accent)"
                : knownStatus === "learning"
                  ? "var(--status-learning)"
                  : "var(--ink-soft)",
          }}
        >
          {knownStatus ?? "new"}
        </span>
      </header>

      {glosses.length > 0 ? (
        <ul style={{ margin: 0, paddingLeft: "var(--s-4)", color: "var(--ink)" }}>
          {glosses.map((g, i) => (
            <li key={i}>{g}</li>
          ))}
        </ul>
      ) : (
        <p style={{ color: "var(--ink-faint)", margin: 0 }}>
          No JMdict entry. Try a different selection.
        </p>
      )}

      <div style={{ display: "flex", gap: "var(--s-3)", color: "var(--ink-soft)", flexWrap: "wrap" }}>
        <span style={{ fontSize: "var(--t-meta)" }}>
          JPDB: {freq ? `#${freq.toLocaleString()}` : "—"}
        </span>
        <span style={{ fontSize: "var(--t-meta)" }}>
          POS: {primary?.senses[0]?.pos.join(", ") ?? token.pos}
        </span>
        {toneTags && toneTags.length > 0 && (
          <div style={{ display: "flex", gap: "var(--s-1)", flexWrap: "wrap" }}>
            {toneTags.map((t) => (
              <span
                key={t}
                style={{
                  padding: "0 var(--s-2)",
                  border: "1px solid var(--rule-soft)",
                  borderRadius: "var(--r-pill)",
                  fontSize: "var(--t-meta)",
                  letterSpacing: "var(--track-mono)",
                  color: "var(--accent)",
                }}
              >
                {t}
              </span>
            ))}
          </div>
        )}
      </div>

      <KanjiBreakdown text={lemma} />

      {primary && (primary.senses.flatMap((s) => s.glosses).length ?? 0) > 3 && (
        <Expandable label="All meanings">
          <ul style={{ margin: 0, paddingLeft: "var(--s-4)" }}>
            {primary.senses.flatMap((s, si) =>
              s.glosses.map((g, gi) => (
                <li key={`${si}-${gi}`}>
                  <span style={{ color: "var(--ink-soft)", marginRight: "var(--s-2)" }}>
                    {s.pos.join(",")}
                  </span>
                  {g}
                </li>
              )),
            )}
          </ul>
        </Expandable>
      )}

      <footer style={{ display: "flex", justifyContent: "space-between", gap: "var(--s-2)" }}>
        {onMine && primary && (
          <button
            className="btn btn-primary"
            onClick={() =>
              onMine({
                surface: token.surface,
                lemma,
                reading,
                glosses,
                jm: primary,
              })
            }
          >
            Mine
          </button>
        )}
        <button className="btn" onClick={onClose}>
          Close
        </button>
      </footer>
    </div>
  );
}

function Expandable({ label, children }: { label: string; children: React.ReactNode }) {
  const [open, setOpen] = useState(false);
  return (
    <div>
      <button
        onClick={() => setOpen((o) => !o)}
        style={{
          background: "transparent",
          border: 0,
          padding: 0,
          color: "var(--accent)",
          cursor: "pointer",
          font: "inherit",
        }}
      >
        {open ? "− " : "+ "}
        {label}
      </button>
      {open && <div style={{ marginTop: "var(--s-2)" }}>{children}</div>}
    </div>
  );
}

function isKanji(c: string): boolean {
  const code = c.codePointAt(0) ?? 0;
  return (
    (code >= 0x4e00 && code <= 0x9fff) ||
    (code >= 0x3400 && code <= 0x4dbf)
  );
}

function KanjiBreakdown({ text }: { text: string }) {
  const chars = useMemo(() => Array.from(text).filter(isKanji), [text]);
  const { data } = useQuery({
    queryKey: ["wk-breakdown", text],
    queryFn: async () => {
      const out: { c: string; entry?: WkKanji; siblings: string[] }[] = [];
      for (const c of chars) {
        const entry = await wkKanji(c);
        const siblings = (await wkVocabByKanji(c)).slice(0, 6);
        out.push({ c, entry, siblings });
      }
      return out;
    },
    enabled: chars.length > 0,
  });
  if (chars.length === 0) return null;
  return (
    <Expandable label="Kanji breakdown">
      <ul style={{ margin: 0, padding: 0, listStyle: "none", display: "grid", gap: "var(--s-2)" }}>
        {(data ?? []).map(({ c, entry, siblings }) => (
          <li
            key={c}
            style={{
              display: "grid",
              gridTemplateColumns: "auto 1fr",
              gap: "var(--s-3)",
              alignItems: "baseline",
            }}
          >
            <span style={{ fontFamily: "var(--font-serif, serif)", fontSize: "var(--t-display-sm)" }}>
              {c}
            </span>
            <div>
              {entry ? (
                <>
                  <div>
                    <span style={{ color: "var(--accent)" }}>L{entry.level}</span>
                    {" · "}
                    <span>{entry.meaning}</span>
                    {" · "}
                    <span style={{ color: "var(--ink-soft)" }}>
                      {entry.reading} ({entry.reading_type})
                    </span>
                  </div>
                  {siblings.length > 0 && (
                    <div style={{ color: "var(--ink-soft)", fontSize: "var(--t-meta)" }}>
                      {siblings.join("、")}
                    </div>
                  )}
                </>
              ) : (
                <span style={{ color: "var(--ink-faint)" }}>not in WaniKani</span>
              )}
            </div>
          </li>
        ))}
      </ul>
    </Expandable>
  );
}

function positionAnchored(anchor: DOMRect): { top: number; left: number } {
  const margin = 8;
  const popupW = 360;
  const popupH = 280;
  let top = anchor.bottom + margin;
  let left = anchor.left;
  if (top + popupH > window.innerHeight) top = anchor.top - popupH - margin;
  if (left + popupW > window.innerWidth) left = window.innerWidth - popupW - margin;
  if (left < margin) left = margin;
  return { top, left };
}
