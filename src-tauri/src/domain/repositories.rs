// Los repositorios son internos del crate: `async fn` en traits es válido aquí
// y evita añadir una dependencia de runtime async-trait.
#![allow(async_fn_in_trait)]

use uuid::Uuid;

use super::errors::DomainError;
use super::models::{ClipboardEntry, SearchResult, Workspace};

/// Filtros de paginación/listado compartidos.
#[derive(Debug, Clone)]
pub struct ListQuery {
    pub workspace_id: Option<Uuid>,
    pub content_type: Option<String>,
    pub favorite_only: bool,
    pub pinned_only: bool,
    pub search: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

impl Default for ListQuery {
    fn default() -> Self {
        Self {
            workspace_id: None,
            content_type: None,
            favorite_only: false,
            pinned_only: false,
            search: None,
            limit: 50,
            offset: 0,
        }
    }
}

/// Contrato de persistencia para los elementos capturados.
pub trait ClipRepository: Send + Sync {
    /// Inserta un clip nuevo (ya clasificado y hasheado).
    async fn insert(&self, entry: &ClipboardEntry) -> Result<(), DomainError>;

    /// Devuelve un clip por id.
    async fn find_by_id(&self, id: Uuid) -> Result<ClipboardEntry, DomainError>;

    /// Busca por hash de contenido; usado por el deduplicador.
    async fn find_by_hash(&self, hash: &str) -> Result<Option<ClipboardEntry>, DomainError>;

    /// Lista clips paginados, con filtros opcionales.
    #[allow(async_fn_in_trait)]
    async fn list(&self, query: &ListQuery) -> Result<Vec<ClipboardEntry>, DomainError>;

    /// Incrementa copy_count y refresca last_copied_at.
    async fn record_copy(&self, id: Uuid, now: &str) -> Result<(), DomainError>;

    /// Mueve un clip a otro workspace (None = inbox/inbox).
    async fn move_to_workspace(
        &self,
        id: Uuid,
        workspace_id: Option<Uuid>,
        now: &str,
    ) -> Result<(), DomainError>;

    /// Actualiza flag favorito/pinned.
    async fn set_flags(
        &self,
        id: Uuid,
        favorite: Option<bool>,
        pinned: Option<bool>,
    ) -> Result<(), DomainError>;

    /// Elimina un clip y su índice FTS asociado.
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;

    /// Número total de clips (para diagnostics/benchmarks).
    async fn count(&self) -> Result<i64, DomainError>;

    /// Búsqueda FTS5 con ranking BM25.
    async fn search(&self, query: &str, limit: i64) -> Result<Vec<SearchResult>, DomainError>;
}

/// Contrato de persistencia para workspaces.
pub trait WorkspaceRepository: Send + Sync {
    async fn insert(&self, workspace: &Workspace) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Workspace, DomainError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Workspace>, DomainError>;
    async fn list(&self) -> Result<Vec<Workspace>, DomainError>;
    async fn update(&self, workspace: &Workspace) -> Result<(), DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}