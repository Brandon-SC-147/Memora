use std::sync::Arc;

use crate::domain::errors::DomainError;
use crate::domain::models::SearchResult;
use crate::domain::repositories::ClipRepository;

/// Use case: búsqueda por contenido vía FTS5.
pub struct SearchClips<Repo> {
    repo: Arc<Repo>,
}

impl<Repo: ClipRepository> SearchClips<Repo> {
    pub fn new(repo: Arc<Repo>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, query: &str, limit: i64) -> Result<Vec<SearchResult>, DomainError> {
        if limit <= 0 {
            return Ok(Vec::new());
        }
        self.repo.search(query, limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::errors::DomainError;
    use crate::domain::models::{ClipboardEntry, ContentType, ClipMetadata};
    use crate::domain::value_objects::timestamp;
    use uuid::Uuid;

    struct FakeRepo;

    impl ClipRepository for FakeRepo {
        async fn insert(&self, _entry: &ClipboardEntry) -> Result<(), DomainError> {
            Ok(())
        }
        async fn find_by_id(&self, _id: Uuid) -> Result<ClipboardEntry, DomainError> {
            Err(DomainError::NotFound("none".into()))
        }
        async fn find_by_hash(&self, _hash: &str) -> Result<Option<ClipboardEntry>, DomainError> {
            Ok(None)
        }
        async fn list(
            &self,
            _query: &crate::domain::repositories::ListQuery,
        ) -> Result<Vec<ClipboardEntry>, DomainError> {
            Ok(Vec::new())
        }
        async fn record_copy(&self, _id: Uuid, _now: &str) -> Result<(), DomainError> {
            Ok(())
        }
        async fn move_to_workspace(
            &self,
            _id: Uuid,
            _workspace_id: Option<Uuid>,
            _now: &str,
        ) -> Result<(), DomainError> {
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
            Ok(0)
        }
        async fn search(
            &self,
            query: &str,
            _limit: i64,
        ) -> Result<Vec<SearchResult>, DomainError> {
            if query.is_empty() {
                return Ok(Vec::new());
            }
            let now = timestamp::now();
            Ok(vec![SearchResult {
                entry: ClipboardEntry {
                    id: Default::default(),
                    content: query.to_string(),
                    content_type: ContentType::Text,
                    metadata: ClipMetadata::new(),
                    content_hash: "x".into(),
                    is_image: false,
                    image_path: None,
                    copy_count: 1,
                    favorite: false,
                    pinned: false,
                    is_secret: false,
                    created_at: now.clone(),
                    updated_at: now.clone(),
                    last_copied_at: now,
                    workspace_id: None,
                },
                rank: 1.0,
            }])
        }
    }

    #[tokio::test]
    async fn search_requires_repo_impl() {
        let uc = SearchClips::new(Arc::new(FakeRepo));
        let results = uc.execute("sql", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.content, "sql");

        let empty = uc.execute("", 10).await.unwrap();
        assert!(empty.is_empty());
    }

    #[tokio::test]
    async fn non_positive_limit_returns_empty() {
        let uc = SearchClips::new(Arc::new(FakeRepo));
        assert!(uc.execute("sql", 0).await.unwrap().is_empty());
    }
}