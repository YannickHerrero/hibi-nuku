import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/watch/$videoId")({
  component: WatchPage,
});

function WatchPage() {
  const { videoId } = Route.useParams();
  return (
    <div>
      <p style={{ color: "var(--ink-soft)" }}>
        Player for video #{videoId} — wired in Phase 13.
      </p>
    </div>
  );
}
