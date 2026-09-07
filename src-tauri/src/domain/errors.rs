/// Errores de dominio. Todos los errores de la capa de dominio
/// viven aquí; las capas superiores los mapean a errores de Tauri.
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("contenido vacío o inválido: {0}")]
    InvalidContent(String),

    #[error("entidad no encontrada con id {0}")]
    NotFound(String),

    #[error("conflicto de integridad: {0}")]
    Conflict(String),

    #[error("hash de contenido inválido: {0}")]
    InvalidHash(String),

    #[error("error de persistencia: {0}")]
    Persistence(#[from] sqlx::Error),

    #[error("error de serialización: {0}")]
    Serialization(#[from] serde_json::Error),
}