-- Path to a user-uploaded sidecar subtitle file (.srt/.ass/.vtt).
-- When set, the import pipeline reads this file directly instead of
-- extracting a subtitle stream from the video container with ffmpeg.

ALTER TABLE videos ADD COLUMN subtitle_sidecar_path TEXT;
