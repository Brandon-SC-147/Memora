use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

/// Abre el pool de SQLite con WAL + busy_timeout.
///
/// La aplicación usa un pool pequeño (1-2 conexiones): escrituras en una
/// conexión dedicada y lecturas en la otra. Para un desktop app local es
/// más que suficiente y evita contención.
pub fn connect(database_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_path)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .busy_timeout(std::time::Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect_lazy_with(options);

    Ok(pool)
}

/// Aplica las migraciones embebidas (síncrono, bloqueante al arranque).
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!().run(pool).await
}

/// Inicializa la base de datos completa. Devuelve el pool listo.
pub async fn init(database_path: &str) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let pool = connect(database_path)?;
    run_migrations(&pool).await?;
    Ok(pool)
}

/// Pool tipado para consultas directas en comandos/diagnostics.
#[derive(Clone)]
pub struct Db {
    pub pool: SqlitePool,
}

impl Db {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

/// Ruta por defecto de la base de datos dentro del directorio de datos de la app.
pub fn default_database_path(app_data_dir: &std::path::Path) -> std::path::PathBuf {
    app_data_dir.join("memora.db")
}

/// Tipo de fila simple para queries de conteo.
#[derive(Debug)]
pub struct CountRow {
    pub count: i64,
}

/// Conecta a una base de datos en memoria (para tests de integración).
#[cfg(test)]
pub fn in_memory_pool() -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(":memory:")?
        .journal_mode(SqliteJournalMode::Memory)
        .foreign_keys(true);
    Ok(SqlitePoolOptions::new()
        .max_connections(1)
        .connect_lazy_with(options))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn migrations_run_cleanly() {
        let pool = in_memory_pool().unwrap();
        run_migrations(&pool).await.unwrap();

        // Verificar que las tablas esperadas existen.
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name IN ('clips','workspaces','tags','clips_fts','activity_log') ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        let tables: Vec<String> = rows.iter().map(|r| r.0.clone()).collect();
        assert_eq!(
            tables,
            vec![
                "activity_log".to_string(),
                "clips".to_string(),
                "clips_fts".to_string(),
                "tags".to_string(),
                "workspaces".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn fts5_virtual_table_exists() {
        let pool = in_memory_pool().unwrap();
        run_migrations(&pool).await.unwrap();
        let row: (i64,) = sqlx::query_as("SELECT count(*) FROM pragma_table_info('clips_fts')")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(row.0 >= 2); // content + content_type
    }
}