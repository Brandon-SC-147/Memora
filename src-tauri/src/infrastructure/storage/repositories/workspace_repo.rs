use uuid::Uuid;

use crate::domain::errors::DomainError;
use crate::domain::models::Workspace;
use crate::domain::repositories::WorkspaceRepository;
use ::sqlx::SqlitePool;

/// Implementación del repositorio de workspaces sobre SQLite.
#[derive(Clone)]
pub struct SqlxWorkspaceRepository {
    db: SqlitePool,
}

impl SqlxWorkspaceRepository {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

fn row_to_workspace(
    row: &(String, String, Option<String>, Option<String>, String, i64),
) -> Workspace {
    let (id, name, description, icon, created_at, is_default) = row;
    Workspace {
        id: Uuid::parse_str(id).unwrap_or_default(),
        name: name.clone(),
        description: description.clone(),
        icon: icon.clone(),
        created_at: created_at.clone(),
        is_default: *is_default != 0,
    }
}

impl WorkspaceRepository for SqlxWorkspaceRepository {
    async fn insert(&self, workspace: &Workspace) -> Result<(), DomainError> {
        ::sqlx::query(
            "INSERT OR IGNORE INTO workspaces (id, name, description, icon, created_at, is_default) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(workspace.id.to_string())
        .bind(&workspace.name)
        .bind(&workspace.description)
        .bind(&workspace.icon)
        .bind(&workspace.created_at)
        .bind(workspace.is_default as i64)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Workspace, DomainError> {
        let row: (String, String, Option<String>, Option<String>, String, i64) =
            ::sqlx::query_as(
                "SELECT id, name, description, icon, created_at, is_default FROM workspaces WHERE id = ?",
            )
            .bind(id.to_string())
            .fetch_one(&self.db)
            .await?;
        Ok(row_to_workspace(&row))
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Workspace>, DomainError> {
        let row: Option<(String, String, Option<String>, Option<String>, String, i64)> =
            ::sqlx::query_as(
                "SELECT id, name, description, icon, created_at, is_default FROM workspaces WHERE name = ?",
            )
            .bind(name)
            .fetch_optional(&self.db)
            .await?;
        Ok(row.as_ref().map(row_to_workspace))
    }

    async fn list(&self) -> Result<Vec<Workspace>, DomainError> {
        let rows: Vec<(String, String, Option<String>, Option<String>, String, i64)> =
            ::sqlx::query_as(
                "SELECT id, name, description, icon, created_at, is_default FROM workspaces ORDER BY created_at ASC",
            )
            .fetch_all(&self.db)
            .await?;
        Ok(rows.iter().map(row_to_workspace).collect())
    }

    async fn update(&self, workspace: &Workspace) -> Result<(), DomainError> {
        ::sqlx::query(
            "UPDATE workspaces SET name = ?, description = ?, icon = ? WHERE id = ?",
        )
        .bind(&workspace.name)
        .bind(&workspace.description)
        .bind(&workspace.icon)
        .bind(workspace.id.to_string())
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        ::sqlx::query("DELETE FROM workspaces WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.db)
            .await?;
        Ok(())
    }
}