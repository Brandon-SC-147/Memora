use std::sync::Arc;

use sqlx::SqlitePool;

use crate::application::save_clip::SaveClip;
use crate::application::search_clips::SearchClips;
use crate::application::workspaces::WorkspaceService;
use crate::infrastructure::storage::db;
use crate::infrastructure::storage::repositories::{SqlxClipRepository, SqlxWorkspaceRepository};

/// Estado compartido de la aplicación, inyectado en `tauri::Builder` con `manage()`.
///
/// Contiene las dependencias necesarias para los comandos. Los use cases se
/// montan una sola vez aquí para no repetir la composición en cada comando.
#[derive(Clone)]
pub struct AppState {
    pub db: db::Db,
    pub clips: Arc<SqlxClipRepository>,
    pub workspaces: Arc<SqlxWorkspaceRepository>,
    pub save_clip: Arc<SaveClip<SqlxClipRepository>>,
    pub search_clips: Arc<SearchClips<SqlxClipRepository>>,
    pub workspace_service: Arc<WorkspaceService<SqlxWorkspaceRepository>>,
}

impl AppState {
    pub fn new(pool: SqlitePool) -> Self {
        let clips = Arc::new(SqlxClipRepository::new(pool.clone()));
        let workspaces = Arc::new(SqlxWorkspaceRepository::new(pool.clone()));

        Self {
            db: db::Db::new(pool),
            save_clip: Arc::new(SaveClip::new(clips.clone())),
            search_clips: Arc::new(SearchClips::new(clips.clone())),
            workspace_service: Arc::new(WorkspaceService::new(workspaces.clone())),
            clips,
            workspaces,
        }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.db.pool
    }
}