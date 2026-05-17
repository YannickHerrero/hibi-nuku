use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{Row, SqlitePool};

use super::model::{Video, VideoStatus};

pub struct NewVideo {
    pub path: String,
    pub title: String,
    pub source_tag: String,
    pub duration_ms: i64,
    pub jp_audio_idx: Option<i64>,
    pub jp_subtitle_idx: Option<i64>,
    pub subtitle_format: Option<String>,
    pub status: VideoStatus,
}

pub async fn insert(pool: &SqlitePool, new: NewVideo) -> Result<i64> {
    let now = Utc::now().to_rfc3339();
    let status = new.status.as_str();
    let id = sqlx::query(
        r#"
        INSERT INTO videos
          (path, title, source_tag, duration_ms,
           jp_audio_idx, jp_subtitle_idx, subtitle_format,
           status, imported_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&new.path)
    .bind(&new.title)
    .bind(&new.source_tag)
    .bind(new.duration_ms)
    .bind(new.jp_audio_idx)
    .bind(new.jp_subtitle_idx)
    .bind(new.subtitle_format.as_deref())
    .bind(status)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .context("insert video")?
    .last_insert_rowid();
    Ok(id)
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<Video>> {
    let rows = sqlx::query(
        r#"
        SELECT id, path, title, source_tag, duration_ms,
               jp_audio_idx, jp_subtitle_idx, subtitle_format,
               status, error_message, imported_at, updated_at
        FROM videos
        ORDER BY imported_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .context("list videos")?;
    rows.into_iter().map(row_to_video).collect()
}

pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<Video>> {
    let row = sqlx::query(
        r#"
        SELECT id, path, title, source_tag, duration_ms,
               jp_audio_idx, jp_subtitle_idx, subtitle_format,
               status, error_message, imported_at, updated_at
        FROM videos WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .context("get video")?;
    row.map(row_to_video).transpose()
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool> {
    let res = sqlx::query("DELETE FROM videos WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .context("delete video")?;
    Ok(res.rows_affected() > 0)
}

pub async fn update_metadata(
    pool: &SqlitePool,
    id: i64,
    title: Option<&str>,
    source_tag: Option<&str>,
) -> Result<bool> {
    let now = Utc::now().to_rfc3339();
    let res = sqlx::query(
        r#"
        UPDATE videos
        SET title      = COALESCE(?, title),
            source_tag = COALESCE(?, source_tag),
            updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(title)
    .bind(source_tag)
    .bind(&now)
    .bind(id)
    .execute(pool)
    .await
    .context("update video metadata")?;
    Ok(res.rows_affected() > 0)
}

pub async fn set_status(
    pool: &SqlitePool,
    id: i64,
    status: VideoStatus,
    error_message: Option<&str>,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        UPDATE videos
        SET status = ?, error_message = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(status.as_str())
    .bind(error_message)
    .bind(&now)
    .bind(id)
    .execute(pool)
    .await
    .context("set status")?;
    Ok(())
}

pub async fn set_tracks(
    pool: &SqlitePool,
    id: i64,
    duration_ms: i64,
    jp_audio_idx: Option<i64>,
    jp_subtitle_idx: Option<i64>,
    subtitle_format: Option<&str>,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        UPDATE videos
        SET duration_ms = ?, jp_audio_idx = ?, jp_subtitle_idx = ?,
            subtitle_format = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(duration_ms)
    .bind(jp_audio_idx)
    .bind(jp_subtitle_idx)
    .bind(subtitle_format)
    .bind(&now)
    .bind(id)
    .execute(pool)
    .await
    .context("set tracks")?;
    Ok(())
}

fn row_to_video(row: sqlx::sqlite::SqliteRow) -> Result<Video> {
    use chrono::DateTime;
    let status_str: String = row.try_get("status")?;
    let imported_at: String = row.try_get("imported_at")?;
    let updated_at: String = row.try_get("updated_at")?;
    Ok(Video {
        id: row.try_get("id")?,
        path: row.try_get("path")?,
        title: row.try_get("title")?,
        source_tag: row.try_get("source_tag")?,
        duration_ms: row.try_get("duration_ms")?,
        jp_audio_idx: row.try_get("jp_audio_idx")?,
        jp_subtitle_idx: row.try_get("jp_subtitle_idx")?,
        subtitle_format: row.try_get("subtitle_format")?,
        status: VideoStatus::parse(&status_str)
            .with_context(|| format!("unknown status `{status_str}`"))?,
        error_message: row.try_get("error_message")?,
        imported_at: DateTime::parse_from_rfc3339(&imported_at)?.with_timezone(&chrono::Utc),
        updated_at: DateTime::parse_from_rfc3339(&updated_at)?.with_timezone(&chrono::Utc),
    })
}
