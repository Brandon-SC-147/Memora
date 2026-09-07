use std::sync::Arc;

use tauri::State;
use uuid::Uuid;

use crate::domain::models::{ClipboardEntry, Workspace};
use crate::domain::repositories::{ClipRepository, WorkspaceRepository};
use crate::infrastructure::state::AppState;

/// Información de la app + salud de la base de datos (para Diagnostics).
#[tauri::command]
pub async fn app_info() -> Result<AppInfo, String> {
    Ok(AppInfo {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    name: String,
    version: String,
}

/// Estado de salud para Diagnostics.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthInfo {
    app: AppInfo,
    database_healthy: bool,
    clip_count: i64,
    workspace_count: i64,
}

#[tauri::command]
pub async fn health(state: State<'_, Arc<AppState>>) -> Result<HealthInfo, String> {
    let clip_count = state.clips.count().await.map_err(|e| e.to_string())?;
    let workspace_count = state
        .workspaces
        .list()
        .await
        .map(|ws| ws.len() as i64)
        .map_err(|e| e.to_string())?;

    Ok(HealthInfo {
        app: AppInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        database_healthy: true,
        clip_count,
        workspace_count,
    })
}

/// Lista clips paginados.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListClipsParams {
    pub workspace_id: Option<String>,
    pub content_type: Option<String>,
    pub favorite_only: bool,
    pub pinned_only: bool,
    pub limit: i64,
    pub offset: i64,
}

#[tauri::command]
pub async fn list_clips(
    state: State<'_, Arc<AppState>>,
    params: ListClipsParams,
) -> Result<Vec<ClipboardEntry>, String> {
    let query = crate::domain::repositories::ListQuery {
        workspace_id: params.workspace_id.and_then(|w| Uuid::parse_str(&w).ok()),
        content_type: params.content_type,
        favorite_only: params.favorite_only,
        pinned_only: params.pinned_only,
        search: None,
        limit: params.limit,
        offset: params.offset,
    };
    state.clips.list(&query).await.map_err(|e| e.to_string())
}

/// Búsqueda FTS5.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchParams {
    pub query: String,
    pub limit: Option<i64>,
}

#[tauri::command]
pub async fn search_clips(
    state: State<'_, Arc<AppState>>,
    params: SearchParams,
) -> Result<Vec<crate::domain::models::SearchResult>, String> {
    let limit = params.limit.unwrap_or(50);
    state
        .search_clips
        .execute(&params.query, limit)
        .await
        .map_err(|e| e.to_string())
}

/// Crea un workspace.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkspaceParams {
    pub name: String,
    pub description: Option<String>,
}

#[tauri::command]
pub async fn create_workspace(
    state: State<'_, Arc<AppState>>,
    params: CreateWorkspaceParams,
) -> Result<Workspace, String> {
    state
        .workspace_service
        .create(&params.name, params.description.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// Lista workspaces.
#[tauri::command]
pub async fn list_workspaces(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<Workspace>, String> {
    state
        .workspace_service
        .list()
        .await
        .map_err(|e| e.to_string())
}

/// Renombra un workspace.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameWorkspaceParams {
    pub id: String,
    pub name: String,
}

#[tauri::command]
pub async fn rename_workspace(
    state: State<'_, Arc<AppState>>,
    params: RenameWorkspaceParams,
) -> Result<Workspace, String> {
    let id = Uuid::parse_str(&params.id).map_err(|_| "id inválido".to_string())?;
    state
        .workspace_service
        .rename(id, &params.name)
        .await
        .map_err(|e| e.to_string())
}

/// Elimina un workspace.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteWorkspaceParams {
    pub id: String,
}

#[tauri::command]
pub async fn delete_workspace(
    state: State<'_, Arc<AppState>>,
    params: DeleteWorkspaceParams,
) -> Result<(), String> {
    let id = Uuid::parse_str(&params.id).map_err(|_| "id inválido".to_string())?;
    state
        .workspace_service
        .delete(id)
        .await
        .map_err(|e| e.to_string())
}