import {
  Outlet,
  Link,
  createRootRoute,
  useRouterState,
} from "@tanstack/react-router";
import { InstallPrompt } from "@/components/InstallPrompt";

export const Route = createRootRoute({
  component: RootLayout,
});

function RootLayout() {
  const pathname = useRouterState({ select: (s) => s.location.pathname });
  // The watch page is full-bleed for vertical space — hide the chrome.
  const minimal = pathname.startsWith("/watch/");

  if (minimal) {
    return (
      <div className="min-h-full">
        <Outlet />
        <InstallPrompt />
      </div>
    );
  }

  return (
    <div className="min-h-full">
      <header style={{ borderBottom: "1px solid var(--rule-soft)" }}>
        <div
          className="container-page"
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: "var(--s-3) var(--gutter)",
          }}
        >
          <Link
            to="/"
            style={{
              fontFamily: "var(--font-serif, serif)",
              fontSize: "var(--t-display-sm)",
              letterSpacing: "var(--track-tight)",
            }}
          >
            Hibi Nuku
          </Link>
          <nav style={{ display: "flex", gap: "var(--s-4)" }}>
            <Link to="/" className="link">
              Library
            </Link>
            <Link to="/settings" className="link">
              Settings
            </Link>
          </nav>
        </div>
      </header>
      <main className="container-page" style={{ padding: "var(--s-5) 0" }}>
        <Outlet />
      </main>
      <InstallPrompt />
    </div>
  );
}
