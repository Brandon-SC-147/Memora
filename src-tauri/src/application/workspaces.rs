use std::sync::Arc;

use uuid::Uuid;

use crate::domain::errors::DomainError;
use crate::domain::models::Workspace;
use crate::domain::repositories::WorkspaceRepository;
use crate::domain::value_objects::timestamp;

/// Use case: gestión de workspaces.
pub struct WorkspaceService<Repo> {
    repo: Arc<Repo>,
}

impl<Repo: WorkspaceRepository> WorkspaceService<Repo> {
    pub fn new(repo: Arc<Repo>) -> Self {
        Self { repo }
    }

    pub async fn create(&self, name: &str, description: Option<&str>) -> Result<Workspace, DomainError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(DomainError::InvalidContent("el nombre está vacío".into()));
        }
        if self.repo.find_by_name(name).await?.is_some() {
            return Err(DomainError::Conflict(format!(
                "ya existe un workspace llamado '{name}'"
            )));
        }

        let workspace = Workspace {
            id: Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)),
            name: name.to_string(),
            description: description.map(str::to_string),
            icon: None,
            created_at: timestamp::now(),
            is_default: false,
        };

        self.repo.insert(&workspace).await?;
        Ok(workspace)
    }

    pub async fn list(&self) -> Result<Vec<Workspace>, DomainError> {
        self.repo.list().await
    }

    pub async fn rename(&self, id: Uuid, new_name: &str) -> Result<Workspace, DomainError> {
        let mut workspace = self.repo.find_by_id(id).await?;
        workspace.name = new_name.trim().to_string();
        if workspace.name.is_empty() {
            return Err(DomainError::InvalidContent("el nombre está vacío".into()));
        }
        self.repo.update(&workspace).await?;
        Ok(workspace)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        self.repo.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct InMemoryRepo {
        items: std::sync::Mutex<Vec<Workspace>>,
    }

    impl WorkspaceRepository for InMemoryRepo {
        async fn insert(&self, workspace: &Workspace) -> Result<(), DomainError> {
            self.items.lock().unwrap().push(workspace.clone());
            Ok(())
        }
        async fn find_by_id(&self, id: Uuid) -> Result<Workspace, DomainError> {
            self.items
                .lock()
                .unwrap()
                .iter()
                .find(|w| w.id == id)
                .cloned()
                .ok_or_else(|| DomainError::NotFound(id.to_string()))
        }
        async fn find_by_name(&self, name: &str) -> Result<Option<Workspace>, DomainError> {
            Ok(self
                .items
                .lock()
                .unwrap()
                .iter()
                .find(|w| w.name == name)
                .cloned())
        }
        async fn list(&self) -> Result<Vec<Workspace>, DomainError> {
            Ok(self.items.lock().unwrap().clone())
        }
        async fn update(&self, workspace: &Workspace) -> Result<(), DomainError> {
            let mut guard = self.items.lock().unwrap();
            if let Some(item) = guard.iter_mut().find(|w| w.id == workspace.id) {
                *item = workspace.clone();
            }
            Ok(())
        }
        async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
            self.items.lock().unwrap().retain(|w| w.id != id);
            Ok(())
        }
    }

    #[tokio::test]
    async fn create_workspace() {
        let repo = Arc::new(InMemoryRepo::default());
        let svc = WorkspaceService::new(repo.clone());

        let ws = svc.create("Universidad", None).await.unwrap();
        assert_eq!(ws.name, "Universidad");
        assert_eq!(repo.list().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn duplicate_name_rejected() {
        let repo = Arc::new(InMemoryRepo::default());
        let svc = WorkspaceService::new(repo.clone());

        svc.create("Dev", None).await.unwrap();
        let err = svc.create("Dev", None).await.unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[tokio::test]
    async fn empty_name_rejected() {
        let repo = Arc::new(InMemoryRepo::default());
        let svc = WorkspaceService::new(repo);

        let err = svc.create("   ", None).await.unwrap_err();
        assert!(matches!(err, DomainError::InvalidContent(_)));
    }

    #[tokio::test]
    async fn rename_updates_name() {
        let repo = Arc::new(InMemoryRepo::default());
        let svc = WorkspaceService::new(repo.clone());

        let ws = svc.create("Old", None).await.unwrap();
        let renamed = svc.rename(ws.id, "New").await.unwrap();
        assert_eq!(renamed.name, "New");
    }

    #[tokio::test]
    async fn delete_removes_workspace() {
        let repo = Arc::new(InMemoryRepo::default());
        let svc = WorkspaceService::new(repo.clone());

        let ws = svc.create("Temp", None).await.unwrap();
        svc.delete(ws.id).await.unwrap();
        assert!(repo.list().await.unwrap().is_empty());
    }
}