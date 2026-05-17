import { useMemo, useState } from "react";
import type { SubtitleLine, Token } from "@/api/types";
import type { KnownWord } from "@/api/types";

interface Props {
  line: SubtitleLine | null;
  known: KnownWord[];
  onTokenClick: (token: Token, rect: DOMRect) => void;
}

/// Subtitle strip rendered below the video. Two stacked rows:
///   1. JP line as clickable tokens, underlined by known-words status.
///   2. English translation, blurred until hovered (so you can self-test
///      without spoiling).
/// Hover does nothing for the popup; tap/click only.
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
        padding: "var(--s-4)",
        minHeight: "calc(var(--s-8) + var(--s-2))",
        display: "grid",
        gap: "var(--s-2)",
        justifyItems: "center",
      }}
    >
      {!line ? (
        <span style={{ color: "var(--ink-faint)" }}>—</span>
      ) : (
        <>
          <div
            lang="ja"
            style={{
              fontSize: "clamp(20px, 2.6vw, 30px)",
              lineHeight: 1.5,
              maxWidth: "100%",
              textAlign: "center",
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
          <BlurredTranslation text={line.translation} />
        </>
      )}
    </div>
  );
}

function BlurredTranslation({ text }: { text: string | null }) {
  const [revealed, setRevealed] = useState(false);
  if (!text) return null;
  return (
    <div
      onMouseEnter={() => setRevealed(true)}
      onMouseLeave={() => setRevealed(false)}
      onClick={() => setRevealed((v) => !v)}
      style={{
        color: "var(--ink-soft)",
        fontSize: "var(--t-body-sm)",
        maxWidth: "min(720px, 100%)",
        textAlign: "center",
        cursor: "pointer",
        // The whole row is the hover target; the inner text carries
        // the blur so it can be unblurred without re-laying out.
        userSelect: revealed ? "text" : "none",
      }}
      title={revealed ? undefined : "Hover or tap to reveal"}
    >
      <span
        style={{
          filter: revealed ? "none" : "blur(6px)",
          transition: "filter 120ms ease",
        }}
      >
        {text}
      </span>
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
  const style = stylesForStatus(token, status);
  if (token.pos === "punctuation") {
    return <span style={{ color: "var(--ink-faint)" }}>{token.surface}</span>;
  }
  return (
    <span
      onClick={(e) => {
        const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
        onClick(token, rect);
      }}
      style={style}
    >
      {token.surface}
    </span>
  );
}

function stylesForStatus(
  token: Token,
  status: KnownWord["status"] | undefined,
): React.CSSProperties {
  const base: React.CSSProperties = {
    color: "var(--ink)",
    cursor: "pointer",
    padding: "0 1px",
    textDecorationLine: "none",
    textDecorationStyle: "solid",
    textDecorationThickness: "2px",
    textUnderlineOffset: "4px",
  };
  switch (status) {
    case "known":
      // Already known → no decoration, normal ink.
      return base;
    case "learning":
      return {
        ...base,
        textDecorationLine: "underline",
        textDecorationColor: "var(--status-learning)",
        textDecorationThickness: "3px",
      };
    case "ignored":
      // Out of mind → muted, no decoration.
      return { ...base, color: "var(--ink-faint)" };
    default:
      // "new" — solid underline in faint ink. Particles dim a touch
      // so kanji/verb roots stand out from grammatical glue.
      return {
        ...base,
        color: token.pos === "particle" ? "var(--ink-soft)" : "var(--ink)",
        textDecorationLine: "underline",
        textDecorationColor: "var(--ink-faint)",
      };
  }
}

function safeParseTokens(json: string): Token[] {
  try {
    return JSON.parse(json) as Token[];
  } catch {
    return [];
  }
}
