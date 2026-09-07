use std::sync::Arc;

use uuid::Uuid;

use crate::domain::errors::DomainError;
use crate::domain::models::{ClipboardEntry, ContentType, ClipMetadata};
use crate::domain::repositories::ClipRepository;
use crate::domain::value_objects::timestamp;

/// Resultado de guardar un clip: nuevo o duplicado (con contador incrementado).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveOutcome {
    Created(ClipboardEntry),
    Duplicate(ClipboardEntry),
}

/// Use case: guardar contenido del portapapeles.
///
/// Pipeline: clasificación (tipo + metadata) → hash → dedupe → insert o bump.
/// La detección de secretos se compone externamente antes de llamar a este use case.
pub struct SaveClip<Repo> {
    repo: Arc<Repo>,
}

impl<Repo: ClipRepository> SaveClip<Repo> {
    pub fn new(repo: Arc<Repo>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        content: &str,
        content_type: ContentType,
        metadata: ClipMetadata,
        workspace_id: Option<Uuid>,
    ) -> Result<SaveOutcome, DomainError> {
        let content = content.trim();
        if content.is_empty() {
            return Err(DomainError::InvalidContent(
                "el portapapeles está vacío".into(),
            ));
        }

        let hash = crate::domain::value_objects::ContentHash::compute(content).to_string();

        // Dedupe: si ya existe, solo incrementar contador.
        if let Some(existing) = self.repo.find_by_hash(&hash).await? {
            let now = timestamp::now();
            let mut updated = existing.clone();
            updated.record_copy(&now);
            self.repo.record_copy(updated.id, &now).await?;
            return Ok(SaveOutcome::Duplicate(updated));
        }

        let now = timestamp::now();
        let entry = ClipboardEntry {
            id: Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)),
            created_at: now.clone(),
            updated_at: now.clone(),
            last_copied_at: now,
            content_hash: hash,
            content_type,
            metadata,
            workspace_id,
            copy_count: 1,
            favorite: false,
            pinned: false,
            is_secret: false,
            is_image: false,
            image_path: None,
            content: content.to_string(),
        };

        self.repo.insert(&entry).await?;
        Ok(SaveOutcome::Created(entry))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{ClipboardEntry, SearchResult, Workspace};
    use crate::domain::repositories::ListQuery;

    /// Repositorio en memoria para tests de use case (sin infraestructura).
    #[derive(Default)]
    struct InMemoryRepo {
        entries: std::sync::Mutex<Vec<ClipboardEntry>>,
    }

    impl ClipRepository for InMemoryRepo {
        async fn insert(&self, entry: &ClipboardEntry) -> Result<(), DomainError> {
            self.entries.lock().unwrap().push(entry.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<ClipboardEntry, DomainError> {
            self.entries
                .lock()
                .unwrap()
                .iter()
                .find(|e| e.id == id)
                .cloned()
                .ok_or_else(|| DomainError::NotFound(id.to_string()))
        }

        async fn find_by_hash(
            &self,
            hash: &str,
        ) -> Result<Option<ClipboardEntry>, DomainError> {
            Ok(self
                .entries
                .lock()
                .unwrap()
                .iter()
                .find(|e| e.content_hash == hash)
                .cloned())
        }

        async fn list(
            &self,
            _query: &ListQuery,
        ) -> Result<Vec<ClipboardEntry>, DomainError> {
            Ok(self.entries.lock().unwrap().clone())
        }

        async fn record_copy(&self, id: Uuid, now: &str) -> Result<(), DomainError> {
            let mut guard = self.entries.lock().unwrap();
            if let Some(entry) = guard.iter_mut().find(|e| e.id == id) {
                entry.record_copy(now);
            }
            Ok(())
        }

        async fn move_to_workspace(
            &self,
            id: Uuid,
            workspace_id: Option<Uuid>,
            now: &str,
        ) -> Result<(), DomainError> {
            let mut guard = self.entries.lock().unwrap();
            if let Some(entry) = guard.iter_mut().find(|e| e.id == id) {
                entry.move_to_workspace(workspace_id, now);
            }
            Ok(())
        }

        async fn set_flags(
            &self,
            _id: Uuid,
            _favorite: Option<bool>,
            _pinned: Option<bool>,
        ) -> Result<(), DomainError> {
            Ok(())
        }

        async fn delete(&self, _id: Uuid) -> Result<(), DomainError> {
            Ok(())
        }

        async fn count(&self) -> Result<i64, DomainError> {
            Ok(self.entries.lock().unwrap().len() as i64)
        }

        async fn search(
            &self,
            query: &str,
            _limit: i64,
        ) -> Result<Vec<SearchResult>, DomainError> {
            let query = query.to_lowercase();
            Ok(self
                .entries
                .lock()
                .unwrap()
                .iter()
                .filter(|e| e.content.to_lowercase().contains(&query))
                .map(|e| SearchResult {
                    entry: e.clone(),
                    rank: 0.0,
                })
                .collect())
        }
    }

    #[tokio::test]
    async fn first_copy_creates_entry() {
        let repo = Arc::new(InMemoryRepo::default());
        let uc = SaveClip::new(repo.clone());

        let outcome = uc
            .execute(
                "  docker compose up  ",
                ContentType::Command,
                ClipMetadata::new(),
                None,
            )
            .await
            .unwrap();

        match outcome {
            SaveOutcome::Created(entry) => {
                assert_eq!(entry.content, "docker compose up");
                assert_eq!(entry.copy_count, 1);
            }
            SaveOutcome::Duplicate(_) => panic!("debe crear la primera entrada"),
        }
        assert_eq!(repo.count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn repeated_copy_dedupes_and_increments() {
        let repo = Arc::new(InMemoryRepo::default());
        let uc = SaveClip::new(repo.clone());

        let _ = uc
            .execute(
                "docker compose up",
                ContentType::Command,
                ClipMetadata::new(),
                None,
            )
            .await
            .unwrap();
        let second = uc
            .execute(
                "docker compose up",
                ContentType::Command,
                ClipMetadata::new(),
                None,
            )
            .await
            .unwrap();

        match second {
            SaveOutcome::Duplicate(entry) => {
                assert_eq!(entry.copy_count, 2);
            }
            SaveOutcome::Created(_) => panic!("no debe crear un duplicado"),
        }
        assert_eq!(repo.count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn empty_content_is_rejected() {
        let repo = Arc::new(InMemoryRepo::default());
        let uc = SaveClip::new(repo);

        let err = uc
            .execute("   ", ContentType::Text, ClipMetadata::new(), None)
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::InvalidContent(_)));
    }

    #[allow(unused)]
    impl Workspace {
        fn fixture() -> Workspace {
            Workspace {
                id: Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)),
                name: "Dev".into(),
                description: None,
                icon: None,
                created_at: timestamp::now(),
                is_default: false,
            }
        }
    }
}