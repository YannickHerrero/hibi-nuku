-- Path to a pre-remuxed MP4 (faststart, H.264 video stream-copied,
-- AAC audio fallback). When set, /api/videos/:id/stream serves this
-- file directly with native HTTP Range support — no ffmpeg in the
-- request path, and the browser can seek freely.
--
-- When NULL the import pipeline hasn't produced (or hasn't needed
-- to produce) a remuxed copy yet; the stream endpoint falls back to
-- the on-the-fly ffmpeg pipe.

ALTER TABLE videos ADD COLUMN remuxed_path TEXT;
