import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/api/client";
import type { SubtitleLine } from "@/api/types";

interface Props {
  videoId: number;
  line: SubtitleLine;
  focusWord: string;
  focusWordReading: string;
  defaultEnglish: string;
  defaultGrammarNote: string;
  defaultTags?: string[];
  onClose: () => void;
  onMined?: () => void;
}

export function MiningModal(props: Props) {
  const qc = useQueryClient();
  const [sentence, setSentence] = useState(props.line.rawText);
  const [focusWord, setFocusWord] = useState(props.focusWord);
  const [focusWordReading, setFocusWordReading] = useState(props.focusWordReading);
  const [english, setEnglish] = useState(props.defaultEnglish);
  const [grammarNote, setGrammarNote] = useState(props.defaultGrammarNote);
  const [tags, setTags] = useState(props.defaultTags?.join(", ") ?? "");
  const [padBefore, setPadBefore] = useState(500);
  const [padAfter, setPadAfter] = useState(500);

  const mine = useMutation({
    mutationFn: async () => {
      return api.mine({
        videoId: props.videoId,
        lineId: props.line.id,
        focusWord,
        focusWordReading,
        padBeforeMs: padBefore,
        padAfterMs: padAfter,
        userEnglishOverride: english !== props.defaultEnglish ? english : null,
        userGrammarOverride: grammarNote !== props.defaultGrammarNote ? grammarNote : null,
        tags: tags
          .split(",")
          .map((s) => s.trim())
          .filter(Boolean),
      });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["known"] });
      props.onMined?.();
      props.onClose();
    },
  });

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(0,0,0,0.45)",
        display: "grid",
        placeItems: "center",
        zIndex: 60,
      }}
      onClick={props.onClose}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          background: "var(--paper)",
          border: "1px solid var(--ink)",
          padding: "var(--s-5)",
          minWidth: 520,
          maxWidth: "92vw",
          display: "grid",
          gap: "var(--s-3)",
        }}
      >
        <h2 style={{ fontFamily: "var(--font-serif, serif)", margin: 0 }}>
          Mine to Hibi
        </h2>
        <Field label="Sentence">
          <textarea
            rows={2}
            value={sentence}
            onChange={(e) => setSentence(e.target.value)}
            style={inputStyle}
          />
        </Field>
        <Two>
          <Field label="Focus word">
            <input value={focusWord} onChange={(e) => setFocusWord(e.target.value)} style={inputStyle} />
          </Field>
          <Field label="Reading">
            <input
              value={focusWordReading}
              onChange={(e) => setFocusWordReading(e.target.value)}
              style={inputStyle}
            />
          </Field>
        </Two>
        <Field label="English translation">
          <textarea
            rows={2}
            value={english}
            onChange={(e) => setEnglish(e.target.value)}
            style={inputStyle}
          />
        </Field>
        <Field label="Grammar note">
          <textarea
            rows={2}
            value={grammarNote}
            onChange={(e) => setGrammarNote(e.target.value)}
            style={inputStyle}
          />
        </Field>
        <Field label="Tags (comma-separated)">
          <input value={tags} onChange={(e) => setTags(e.target.value)} style={inputStyle} />
        </Field>
        <Two>
          <Field label="Pad before (ms)">
            <input
              type="number"
              value={padBefore}
              onChange={(e) => setPadBefore(Number(e.target.value))}
              style={inputStyle}
            />
          </Field>
          <Field label="Pad after (ms)">
            <input
              type="number"
              value={padAfter}
              onChange={(e) => setPadAfter(Number(e.target.value))}
              style={inputStyle}
            />
          </Field>
        </Two>
        {mine.error && (
          <pre style={errStyle}>{String(mine.error)}</pre>
        )}
        <footer style={{ display: "flex", justifyContent: "flex-end", gap: "var(--s-2)" }}>
          <button className="btn" onClick={props.onClose} disabled={mine.isPending}>
            Cancel
          </button>
          <button
            className="btn btn-primary"
            disabled={mine.isPending || !focusWord || !focusWordReading}
            onClick={() => mine.mutate()}
          >
            {mine.isPending ? "Mining…" : "Create card"}
          </button>
        </footer>
      </div>
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label style={{ display: "grid", gap: "var(--s-1)" }}>
      <span style={{ color: "var(--ink-soft)", fontSize: "var(--t-meta)" }}>{label}</span>
      {children}
    </label>
  );
}

function Two({ children }: { children: React.ReactNode }) {
  return (
    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--s-3)" }}>
      {children}
    </div>
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

const errStyle: React.CSSProperties = {
  color: "#a33",
  background: "var(--paper-alt)",
  padding: "var(--s-3)",
  whiteSpace: "pre-wrap",
  fontSize: "var(--t-meta)",
  margin: 0,
};
