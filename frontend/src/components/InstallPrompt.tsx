import { useEffect, useState } from "react";

interface InstallEvent extends Event {
  prompt: () => Promise<void>;
  userChoice: Promise<{ outcome: "accepted" | "dismissed" }>;
}

export function InstallPrompt() {
  const [evt, setEvt] = useState<InstallEvent | null>(null);
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    const handler = (e: Event) => {
      e.preventDefault();
      setEvt(e as InstallEvent);
    };
    window.addEventListener("beforeinstallprompt", handler);
    return () => window.removeEventListener("beforeinstallprompt", handler);
  }, []);

  if (!evt || dismissed) return null;
  return (
    <div
      style={{
        position: "fixed",
        bottom: "var(--s-4)",
        right: "var(--s-4)",
        background: "var(--paper)",
        border: "1px solid var(--ink)",
        padding: "var(--s-3)",
        display: "flex",
        gap: "var(--s-2)",
        alignItems: "center",
        zIndex: 40,
      }}
    >
      <span style={{ fontSize: "var(--t-meta)" }}>Install Hibi Nuku?</span>
      <button
        className="btn btn-primary"
        onClick={async () => {
          await evt.prompt();
          setDismissed(true);
        }}
      >
        Install
      </button>
      <button
        className="btn btn-ghost"
        onClick={() => setDismissed(true)}
      >
        Later
      </button>
    </div>
  );
}
