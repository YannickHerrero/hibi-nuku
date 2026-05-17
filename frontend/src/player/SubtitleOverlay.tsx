import { useMemo } from "react";
import type { SubtitleLine, Token } from "@/api/types";
import type { KnownWord } from "@/api/types";

interface Props {
  line: SubtitleLine | null;
  known: KnownWord[];
  onTokenClick: (token: Token, rect: DOMRect) => void;
}

export function SubtitleOverlay({ line, known, onTokenClick }: Props) {
  const knownByLemma = useMemo(() => {
    const m = new Map<string, KnownWord["status"]>();
    for (const w of known) m.set(w.lemma, w.status);
    return m;
  }, [known]);

  if (!line) return null;
  const tokens: Token[] = line.tokensJson ? safeParseTokens(line.tokensJson) : [];

  return (
    <div
      style={{
        position: "absolute",
        left: 0,
        right: 0,
        bottom: "8%",
        textAlign: "center",
        pointerEvents: "none",
      }}
    >
      <div
        lang="ja"
        style={{
          display: "inline-block",
          padding: "var(--s-2) var(--s-4)",
          background: "rgba(0,0,0,0.55)",
          color: "white",
          fontSize: "clamp(18px, 3.5vw, 30px)",
          lineHeight: 1.5,
          pointerEvents: "auto",
          maxWidth: "90%",
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
            ? "rgba(255,255,255,0.6)"
            : "white";
  return (
    <span
      onClick={(e) => {
        const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
        onClick(token, rect);
      }}
      onMouseEnter={(e) => {
        const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
        onClick(token, rect);
      }}
      style={{
        color,
        cursor: "pointer",
        padding: "0 1px",
        borderBottom: status === "learning" ? "2px solid var(--status-learning)" : undefined,
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
