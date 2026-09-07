pub mod application;
pub mod domain;
pub mod infrastructure;

use std::sync::Arc;

use infrastructure::commands;
use infrastructure::state::AppState;
use infrastructure::storage::db;

use tracing_subscriber::EnvFilter;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;

            let db_path = db::default_database_path(&data_dir)
                .to_str()
                .map(str::to_string)
                .ok_or_else(|| {
                    std::io::Error::other("ruta de datos no UTF-8")
                })?;

            let pool = tauri::async_runtime::block_on(db::init(&db_path))?;
            let state = Arc::new(AppState::new(pool));
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::health,
            commands::list_clips,
            commands::search_clips,
            commands::create_workspace,
            commands::list_workspaces,
            commands::rename_workspace,
            commands::delete_workspace,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Prueba de arranque del core sin la GUI: inicializa la DB en memoria
/// y verifica que las migraciones y el repositorio básico funcionan.
#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::{ClipMetadata, ContentType};
    use domain::repositories::{ClipRepository, ListQuery};

    #[tokio::test]
    async fn end_to_end_sqlite_flow() {
        let pool = db::in_memory_pool().unwrap();
        db::run_migrations(&pool).await.unwrap();

        let clips = infrastructure::storage::repositories::SqlxClipRepository::new(pool.clone());
        let workspaces =
            infrastructure::storage::repositories::SqlxWorkspaceRepository::new(pool);

        let svc = application::workspaces::WorkspaceService::new(Arc::new(workspaces));
        let dev = svc.create("Desarrollo", None).await.unwrap();

        let save = application::save_clip::SaveClip::new(Arc::new(clips.clone()));
        let outcome = save
            .execute(
                "SELECT * FROM users;",
                ContentType::Code {
                    language: Some("sql".into()),
                },
                ClipMetadata::new().with_field("language", "sql"),
                Some(dev.id),
            )
            .await
            .unwrap();

        let entry = match outcome {
            application::save_clip::SaveOutcome::Created(e) => e,
            _ => panic!("debe crear"),
        };
        assert_eq!(entry.copy_count, 1);

        // Duplicado → incrementa contador, no crea nueva fila.
        let second = save
            .execute(
                "SELECT * FROM users;",
                ContentType::Code {
                    language: Some("sql".into()),
                },
                ClipMetadata::new(),
                None,
            )
            .await
            .unwrap();
        assert!(matches!(
            second,
            application::save_clip::SaveOutcome::Duplicate(_)
        ));

        let listed = clips
            .list(&ListQuery {
                workspace_id: Some(dev.id),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(listed.len(), 1);

        let found = clips.find_by_hash(&entry.content_hash).await.unwrap();
        assert_eq!(found.unwrap().copy_count, 2);
    }

    #[tokio::test]
    async fn fts_search_finds_inserted_clip() {
        let pool = db::in_memory_pool().unwrap();
        db::run_migrations(&pool).await.unwrap();

        let clips = infrastructure::storage::repositories::SqlxClipRepository::new(pool.clone());
        let save = application::save_clip::SaveClip::new(Arc::new(clips.clone()));

        save.execute(
            "Article about software defects",
            ContentType::Text,
            ClipMetadata::new(),
            None,
        )
        .await
        .unwrap();

        let results = clips.search("defects", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].entry.content.starts_with("Article"));
    }
}