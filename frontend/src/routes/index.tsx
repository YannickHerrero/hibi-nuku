import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/")({
  component: LibraryIndex,
});

function LibraryIndex() {
  return (
    <div>
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
      <p style={{ color: "var(--ink-soft)", marginTop: "var(--s-2)" }}>
        Phase 12 wires this view to the backend.
      </p>
    </div>
  );
}
