use serde::{Deserialize, Serialize};

/// Tipos de contenido detectados por el clasificador.
///
/// Se mantiene como enum para tipado estricto en la capa de dominio.
/// Serializa como string plano ("text", "code:rust", ...) — el mismo formato
/// que se persiste en `clips.content_type` — para que el IPC sea trivial.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentType {
    Text,
    Url,
    Email,
    Json,
    PhoneNumber,
    Color,
    Code { language: Option<String> },
    FilePath,
    Command,
    Other,
}

impl ContentType {
    /// Identificador estable para persistencia (columna `content_type`).
    /// La variante `Code` se serializa con su lenguaje en la forma "code:rust".
    pub fn as_str(&self) -> String {
        match self {
            ContentType::Code { language: Some(lang) } => format!("code:{lang}"),
            other => to_str(other),
        }
    }

    /// Reconstruye un `ContentType` desde su forma persistida/serializada.
    /// Desconocidos mapean a `Other` (forward-compatible).
    pub fn parse(raw: &str) -> Self {
        if let Some((prefix, lang)) = raw.split_once(':') {
            if prefix == "code" {
                return ContentType::Code {
                    language: Some(lang.to_string()),
                };
            }
        }
        match raw {
            "text" => ContentType::Text,
            "url" => ContentType::Url,
            "email" => ContentType::Email,
            "json" => ContentType::Json,
            "phone" => ContentType::PhoneNumber,
            "color" => ContentType::Color,
            "file_path" => ContentType::FilePath,
            "command" => ContentType::Command,
            _ => ContentType::Other,
        }
    }
}

impl Serialize for ContentType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.as_str())
    }
}

impl<'de> Deserialize<'de> for ContentType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(ContentType::parse(&s))
    }
}

fn to_str(ct: &ContentType) -> String {
    match ct {
        ContentType::Text => "text".into(),
        ContentType::Url => "url".into(),
        ContentType::Email => "email".into(),
        ContentType::Json => "json".into(),
        ContentType::PhoneNumber => "phone".into(),
        ContentType::Color => "color".into(),
        ContentType::FilePath => "file_path".into(),
        ContentType::Command => "command".into(),
        ContentType::Other => "other".into(),
        ContentType::Code { .. } => "code".into(),
    }
}

/// Metadatos enriquecidos del clip, almacenados como JSON en `clips.metadata`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ClipMetadata {
    /// Campos abiertos específicos del tipo (dominio URL, formato color, lenguaje, app origen...).
    pub fields: serde_json::Map<String, serde_json::Value>,
}

impl ClipMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_field(mut self, key: &str, value: impl Into<serde_json::Value>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.fields.get(key)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.fields)
    }
}

/// Entidad raíz del dominio: un elemento capturado del portapapeles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardEntry {
    pub id: uuid::Uuid,
    pub content: String,
    pub content_type: ContentType,
    pub metadata: ClipMetadata,
    pub content_hash: String,
    pub is_image: bool,
    pub image_path: Option<String>,
    pub copy_count: i64,
    pub favorite: bool,
    pub pinned: bool,
    pub is_secret: bool,
    pub created_at: String,
    pub updated_at: String,
    pub last_copied_at: String,
    pub workspace_id: Option<uuid::Uuid>,
}

impl ClipboardEntry {
    /// Incrementa el contador de copias y refresca `last_copied_at`.
    pub fn record_copy(&mut self, now: &str) {
        self.copy_count += 1;
        self.last_copied_at = now.to_string();
        self.updated_at = now.to_string();
    }

    pub fn move_to_workspace(&mut self, workspace_id: Option<uuid::Uuid>, now: &str) {
        self.workspace_id = workspace_id;
        self.updated_at = now.to_string();
    }
}

impl Default for ClipboardEntry {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)),
            content: String::new(),
            content_type: ContentType::Other,
            metadata: ClipMetadata::new(),
            content_hash: String::new(),
            is_image: false,
            image_path: None,
            copy_count: 1,
            favorite: false,
            pinned: false,
            is_secret: false,
            created_at: String::new(),
            updated_at: String::new(),
            last_copied_at: String::new(),
            workspace_id: None,
        }
    }
}

/// Entidad Workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub created_at: String,
    pub is_default: bool,
}

/// Resultado de búsqueda: entrada + score de ranking FTS5.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub entry: ClipboardEntry,
    pub rank: f64,
}