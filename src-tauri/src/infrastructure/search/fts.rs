use crate::domain::errors::DomainError;

/// Inserta (o reemplaza) una fila en el índice FTS5.
/// Se llama dentro de la misma transacción que inserta el clip (ver ADR-0002).
///
/// `rowid` es el rowid INTEGER de la tabla `clips` (clave de unión con FTS5).
pub async fn index_insert(
    conn: &mut sqlx::sqlite::SqliteConnection,
    rowid: i64,
    content: &str,
    content_type: &str,
) -> Result<(), DomainError> {
    ::sqlx::query("INSERT OR REPLACE INTO clips_fts (rowid, content, content_type) VALUES (?, ?, ?)")
        .bind(rowid)
        .bind(content)
        .bind(content_type)
        .execute(conn)
        .await?;
    Ok(())
}

/// Elimina una fila del índice FTS5 por rowid.
pub async fn index_delete(
    conn: &mut sqlx::sqlite::SqliteConnection,
    rowid: i64,
) -> Result<(), DomainError> {
    ::sqlx::query("DELETE FROM clips_fts WHERE rowid = ?")
        .bind(rowid)
        .execute(conn)
        .await?;
    Ok(())
}

/// Fila de resultado de búsqueda (entrada + rank FTS5).
#[derive(Debug, Clone, ::sqlx::FromRow)]
pub struct SearchRow {
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
    pub rank: f64,
}

/// Busca en FTS5 con ranking BM25.
///
/// El índice guarda `content` y `content_type`. El JOIN lo hacemos sobre el
/// `rowid` INTEGER, que es la clave natural compartida con la tabla `clips`.
pub async fn search_fts(
    db: &sqlx::SqlitePool,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchRow>, DomainError> {
    let term = query.trim();
    if term.is_empty() {
        return Ok(Vec::new());
    }

    let rows: Vec<SearchRow> = ::sqlx::query_as(
        r#"
        SELECT c.id, c.content, c.content_type, c.metadata, c.content_hash, c.copy_count,
               c.favorite, c.pinned, c.created_at, c.updated_at, c.last_copied_at, c.workspace_id,
               rank
        FROM clips_fts
        JOIN clips c ON c.rowid = clips_fts.rowid
        WHERE clips_fts MATCH ?
        ORDER BY rank
        LIMIT ?
        "#,
    )
    .bind(term)
    .bind(limit)
    .fetch_all(db)
    .await?;

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::storage::db;

    use ::sqlx::SqlitePool;

    async fn setup() -> SqlitePool {
        let pool = db::in_memory_pool().unwrap();
        db::run_migrations(&pool).await.unwrap();
        pool
    }

    async fn insert_clip(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        content: &str,
        content_type: &str,
    ) -> i64 {
        let (rowid,): (i64,) = sqlx::query_as(
            r#"INSERT INTO clips (id, content, content_type, metadata, content_hash,
               copy_count, favorite, pinned, is_secret, created_at, updated_at, last_copied_at, workspace_id)
               VALUES (?, ?, ?, '{}', ?, 1, 0, 0, 0, 't', 't', 't', NULL)
               RETURNING rowid"#,
        )
        .bind(uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)).to_string())
        .bind(content)
        .bind(content_type)
        .bind(format!("hash-{content}"))
        .fetch_one(&mut **tx)
        .await
        .unwrap();
        rowid
    }

    #[tokio::test]
    async fn index_and_search_roundtrip() {
        let pool = setup().await;
        let mut tx = pool.begin().await.unwrap();
        let rowid = insert_clip(&mut tx, "docker compose up", "command").await;
        index_insert(&mut *tx, rowid, "docker compose up", "command")
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let results = search_fts(&pool, "docker", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "docker compose up");
        assert!(results[0].rank < 0.0); // BM25 rank negativo mejor
    }

    #[tokio::test]
    async fn empty_query_returns_nothing() {
        let pool = setup().await;
        let results = search_fts(&pool, "   ", 10).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn search_is_prefix_aware() {
        let pool = setup().await;
        let mut tx = pool.begin().await.unwrap();
        let rowid = insert_clip(&mut tx, "keep ansible playbook", "text").await;
        index_insert(&mut *tx, rowid, "keep ansible playbook", "text")
            .await
            .unwrap();
        tx.commit().await.unwrap();

        // FTS5: "ansible" coincide con token completo.
        let results = search_fts(&pool, "ansible", 10).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn delete_keeps_index_consistent() {
        let pool = setup().await;
        let mut tx = pool.begin().await.unwrap();
        let rowid = insert_clip(&mut tx, "hello world", "text").await;
        index_insert(&mut *tx, rowid, "hello world", "text").await.unwrap();
        tx.commit().await.unwrap();

        let before = search_fts(&pool, "hello", 10).await.unwrap();
        assert_eq!(before.len(), 1);

        let mut tx = pool.begin().await.unwrap();
        index_delete(&mut *tx, rowid).await.unwrap();
        tx.commit().await.unwrap();

        let after = search_fts(&pool, "hello", 10).await.unwrap();
        assert!(after.is_empty());
    }
}