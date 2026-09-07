//! Emisión de eventos Tauri hacia el frontend.
//! Se completa en Fase 2 (clipboard-changed) y Fase 4 (secret-detected).

/// Evento: un clip nuevo ha sido capturado/actualizado.
pub const CLIP_BOARD_CHANGED: &str = "clipboard-changed";

/// Evento: contenido sensible detectado, no se guardó automáticamente.
pub const SECRET_DETECTED: &str = "secret-detected";