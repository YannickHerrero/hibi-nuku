import { useMemo } from "react";
import type { SubtitleLine, Token } from "@/api/types";
import type { KnownWord } from "@/api/types";

interface Props {
  line: SubtitleLine | null;
  known: KnownWord[];
  onTokenClick: (token: Token, rect: DOMRect) => void;
}

/// Subtitle strip rendered below the video — not overlayed on the
/// frame. Click a token to open the popup; hover does nothing so the
/// popup doesn't flicker as the eye scans the line.
export function SubtitleOverlay({ line, known, onTokenClick }: Props) {
  const knownByLemma = useMemo(() => {
    const m = new Map<string, KnownWord["status"]>();
    for (const w of known) m.set(w.lemma, w.status);
    return m;
  }, [known]);

  const tokens: Token[] =
    line?.tokensJson ? safeParseTokens(line.tokensJson) : [];

  return (
    <div
      style={{
        background: "var(--paper-alt)",
        borderTop: "1px solid var(--rule-soft)",
        padding: "var(--s-4) var(--s-4)",
        minHeight: "calc(var(--s-7) + var(--s-2))",
        textAlign: "center",
      }}
    >
      {!line ? (
        <span style={{ color: "var(--ink-faint)" }}>—</span>
      ) : (
        <div
          lang="ja"
          style={{
            display: "inline-block",
            fontSize: "clamp(20px, 2.6vw, 30px)",
            lineHeight: 1.5,
            maxWidth: "100%",
          }}
        >
          {tokens.length === 0 ? (
            <span>{line.rawText}</span>
          ) : (
            tokens.map((t, i) => (
              <TokenSpan
                key={i}
                token={t}
                status={knownByLemma.get(t.lemma)}
                onClick={onTokenClick}
              />
            ))
          )}
        </div>
      )}
    </div>
  );
}

function TokenSpan({
  token,
  status,
  onClick,
}: {
  token: Token;
  status: KnownWord["status"] | undefined;
  onClick: (t: Token, rect: DOMRect) => void;
}) {
  const color =
    status === "known"
      ? "var(--status-known)"
      : status === "learning"
        ? "var(--status-learning)"
        : status === "ignored"
          ? "var(--status-ignored)"
          : token.pos === "punctuation"
            ? "var(--ink-faint)"
            : "var(--ink)";
  return (
    <span
      onClick={(e) => {
        const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
        onClick(token, rect);
      }}
      style={{
        color,
        cursor: "pointer",
        padding: "0 1px",
        borderBottom:
          status === "learning"
            ? "2px solid var(--status-learning)"
            : undefined,
      }}
    >
      {token.surface}
    </span>
  );
}

function safeParseTokens(json: string): Token[] {
  try {
    return JSON.parse(json) as Token[];
  } catch {
    return [];
  }
}
