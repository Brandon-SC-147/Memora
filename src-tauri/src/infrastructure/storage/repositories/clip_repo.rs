use uuid::Uuid;

use crate::domain::errors::DomainError;
use crate::domain::models::{ClipMetadata, ClipboardEntry, ContentType, SearchResult};
use crate::domain::repositories::{ClipRepository, ListQuery};
use crate::infrastructure::search::fts::{index_delete, index_insert, search_fts, SearchRow};
use ::sqlx::SqlitePool;

/// Implementación del repositorio de clips sobre SQLite.
#[derive(Clone)]
pub struct SqlxClipRepository {
    db: SqlitePool,
}

impl SqlxClipRepository {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

/// Fila de la tabla `clips` mapeada con FromRow (12 columnas).
#[derive(Debug, Clone, ::sqlx::FromRow)]
pub struct ClipRow {
    pub id: String,
    pub content: String,
    pub content_type: String,
    pub metadata: String,
    pub content_hash: String,
    pub copy_count: i64,
    pub favorite: i64,
    pub pinned: i64,
    pub created_at: String,
    pub updated_at: String,
    pub last_copied_at: String,
    pub workspace_id: Option<String>,
}

impl From<ClipRow> for ClipboardEntry {
    fn from(row: ClipRow) -> Self {
        ClipboardEntry {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            content: row.content,
            content_type: ContentType::parse(&row.content_type),
            metadata: ClipMetadata {
                fields: serde_json::from_str(&row.metadata).unwrap_or_default(),
            },
            content_hash: row.content_hash,
            is_image: false,
            image_path: None,
            copy_count: row.copy_count,
            favorite: row.favorite != 0,
            pinned: row.pinned != 0,
            is_secret: false,
            created_at: row.created_at,
            updated_at: row.updated_at,
            last_copied_at: row.last_copied_at,
            workspace_id: row.workspace_id.as_deref().and_then(|w| Uuid::parse_str(w).ok()),
        }
    }
}

impl ClipRepository for SqlxClipRepository {
    async fn insert(&self, entry: &ClipboardEntry) -> Result<(), DomainError> {
        let content_type = entry.content_type.as_str();
        let metadata = entry.metadata.to_json()?;

        let mut tx = self.db.begin().await?;
        let (rowid,): (i64,) = ::sqlx::query_as(
            r#"
            INSERT INTO clips
                (id, content, content_type, metadata, content_hash, copy_count,
                 favorite, pinned, is_secret, created_at, updated_at, last_copied_at, workspace_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING rowid
            "#,
        )
        .bind(entry.id.to_string())
        .bind(&entry.content)
        .bind(&content_type)
        .bind(&metadata)
        .bind(&entry.content_hash)
        .bind(entry.copy_count)
        .bind(entry.favorite as i64)
        .bind(entry.pinned as i64)
        .bind(entry.is_secret as i64)
        .bind(&entry.created_at)
        .bind(&entry.updated_at)
        .bind(&entry.last_copied_at)
        .bind(entry.workspace_id.map(|w| w.to_string()))
        .fetch_one(&mut *tx)
        .await?;

        index_insert(&mut *tx, rowid, &entry.content, &content_type).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<ClipboardEntry, DomainError> {
        let row: ClipRow = ::sqlx::query_as(
            r#"
            SELECT id, content, content_type, metadata, content_hash, copy_count,
                   favorite, pinned, created_at, updated_at, last_copied_at, workspace_id
            FROM clips WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_one(&self.db)
        .await?;
        Ok(row.into())
    }

    async fn find_by_hash(&self, hash: &str) -> Result<Option<ClipboardEntry>, DomainError> {
        let row: Option<ClipRow> = ::sqlx::query_as(
            r#"
            SELECT id, content, content_type, metadata, content_hash, copy_count,
                   favorite, pinned, created_at, updated_at, last_copied_at, workspace_id
            FROM clips WHERE content_hash = ?
            "#,
        )
        .bind(hash)
        .fetch_optional(&self.db)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn list(&self, query: &ListQuery) -> Result<Vec<ClipboardEntry>, DomainError> {
        let mut builder = ::sqlx::QueryBuilder::<::sqlx::Sqlite>::new(
            "SELECT id, content, content_type, metadata, content_hash, copy_count, \
             favorite, pinned, created_at, updated_at, last_copied_at, workspace_id FROM clips WHERE 1=1",
        );

        if let Some(ws) = query.workspace_id {
            builder.push(" AND workspace_id = ").push_bind(ws.to_string());
        }
        if let Some(ct) = &query.content_type {
            builder.push(" AND content_type = ").push_bind(ct);
        }
        if query.favorite_only {
            builder.push(" AND favorite = 1");
        }
        if query.pinned_only {
            builder.push(" AND pinned = 1");
        }
        builder
            .push(" ORDER BY last_copied_at DESC LIMIT ")
            .push_bind(query.limit)
            .push(" OFFSET ")
            .push_bind(query.offset);

        let rows: Vec<ClipRow> = builder.build_query_as().fetch_all(&self.db).await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn record_copy(&self, id: Uuid, now: &str) -> Result<(), DomainError> {
        ::sqlx::query(
            "UPDATE clips SET copy_count = copy_count + 1, last_copied_at = ?, updated_at = ? WHERE id = ?",
        )
        .bind(now)
        .bind(now)
        .bind(id.to_string())
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn move_to_workspace(
        &self,
        id: Uuid,
        workspace_id: Option<Uuid>,
        now: &str,
    ) -> Result<(), DomainError> {
        ::sqlx::query("UPDATE clips SET workspace_id = ?, updated_at = ? WHERE id = ?")
            .bind(workspace_id.map(|w| w.to_string()))
            .bind(now)
            .bind(id.to_string())
            .execute(&self.db)
            .await?;
        Ok(())
    }

    async fn set_flags(
        &self,
        id: Uuid,
        favorite: Option<bool>,
        pinned: Option<bool>,
    ) -> Result<(), DomainError> {
        if let Some(f) = favorite {
            ::sqlx::query("UPDATE clips SET favorite = ? WHERE id = ?")
                .bind(f as i64)
                .bind(id.to_string())
                .execute(&self.db)
                .await?;
        }
        if let Some(p) = pinned {
            ::sqlx::query("UPDATE clips SET pinned = ? WHERE id = ?")
                .bind(p as i64)
                .bind(id.to_string())
                .execute(&self.db)
                .await?;
        }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        let mut tx = self.db.begin().await?;
        let (rowid,): (Option<i64>,) = ::sqlx::query_as("SELECT rowid FROM clips WHERE id = ?")
            .bind(id.to_string())
            .fetch_one(&mut *tx)
            .await?;
        if let Some(rowid) = rowid {
            index_delete(&mut *tx, rowid).await?;
        }
        ::sqlx::query("DELETE FROM clips WHERE id = ?")
            .bind(id.to_string())
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn count(&self) -> Result<i64, DomainError> {
        let (count,): (i64,) = ::sqlx::query_as("SELECT count(*) FROM clips")
            .fetch_one(&self.db)
            .await?;
        Ok(count)
    }

    async fn search(&self, query: &str, limit: i64) -> Result<Vec<SearchResult>, DomainError> {
        let rows: Vec<SearchRow> = search_fts(&self.db, query, limit).await?;
        Ok(rows
            .into_iter()
            .map(|r| SearchResult {
                entry: ClipRow {
                    id: r.id,
                    content: r.content,
                    content_type: r.content_type,
                    metadata: r.metadata,
                    content_hash: r.content_hash,
                    copy_count: r.copy_count,
                    favorite: r.favorite,
                    pinned: r.pinned,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    last_copied_at: r.last_copied_at,
                    workspace_id: r.workspace_id,
                }
                .into(),
                rank: r.rank,
            })
            .collect())
    }
}