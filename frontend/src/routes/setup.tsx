import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { setToken } from "@/lib/token";

export const Route = createFileRoute("/setup")({
  component: SetupPage,
});

function SetupPage() {
  const [token, setTok] = useState("");
  const navigate = useNavigate();
  return (
    <div
      style={{
        maxWidth: 480,
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
        Welcome
      </h1>
      <p style={{ color: "var(--ink-soft)", margin: 0 }}>
        Paste your NUKU_TOKEN. It's the bearer set on the server in
        <code style={{ fontFamily: "var(--font-mono, monospace)" }}> .env</code>.
      </p>
      <input
        type="password"
        autoFocus
        value={token}
        onChange={(e) => setTok(e.target.value)}
        placeholder="NUKU_TOKEN"
        style={{
          padding: "var(--s-3) var(--s-4)",
          background: "var(--paper-alt)",
          color: "var(--ink)",
          border: "1px solid var(--rule-soft)",
          fontFamily: "var(--font-mono, monospace)",
        }}
      />
      <button
        className="btn btn-primary"
        disabled={!token}
        onClick={() => {
          setToken(token);
          navigate({ to: "/" });
        }}
      >
        Continue
      </button>
    </div>
  );
}
