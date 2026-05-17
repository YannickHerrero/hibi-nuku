import React from "react";

interface State {
  error: Error | null;
}

export class ErrorBoundary extends React.Component<
  { children: React.ReactNode },
  State
> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    // eslint-disable-next-line no-console
    console.error("ErrorBoundary", error, info);
  }

  render() {
    if (this.state.error) {
      return (
        <div
          style={{
            padding: "var(--s-5)",
            maxWidth: 720,
            margin: "var(--s-7) auto",
          }}
        >
          <h1
            style={{
              fontFamily: "var(--font-serif, serif)",
              fontSize: "var(--t-display-md)",
              margin: 0,
            }}
          >
            Something broke
          </h1>
          <pre
            style={{
              marginTop: "var(--s-3)",
              padding: "var(--s-3)",
              background: "var(--paper-alt)",
              whiteSpace: "pre-wrap",
              fontFamily: "var(--font-mono, monospace)",
              fontSize: "var(--t-meta)",
            }}
          >
            {this.state.error.message}
          </pre>
          <button
            className="btn btn-primary"
            style={{ marginTop: "var(--s-3)" }}
            onClick={() => window.location.reload()}
          >
            Reload
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
