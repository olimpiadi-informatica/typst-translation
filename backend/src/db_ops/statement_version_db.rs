use common::error::Error;
use common::statement_version::StatementVersion;
use sqlx::types::Json;
use sqlx::{Executor, Sqlite};

use crate::db_ops::DatabaseOps;

impl DatabaseOps for StatementVersion {
    async fn insert<'e, E>(&mut self, executor: E) -> Result<(), Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let content_manifest_json = Json(&self.content_manifest.0);
        let row = sqlx::query!(
            r###" 
            INSERT INTO statement_versions (
                task_id,
                version_hash,
                content_manifest,
                is_live,
                created_at
            )
            VALUES (?, ?, ?, ?, ?)
            RETURNING id
            "###,
            self.task_id,
            self.version_hash,
            content_manifest_json, // Use the let binding
            self.is_live,
            self.created_at,
        )
        .fetch_one(executor)
        .await?;
        self.id = row.id;
        Ok(())
    }

    async fn update<'e, E>(&self, executor: E) -> Result<(), Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let content_manifest_json = Json(&self.content_manifest.0);
        sqlx::query!(
            r###" 
            UPDATE statement_versions
            SET
                task_id = ?,
                version_hash = ?,
                content_manifest = ?,
                is_live = ?,
                created_at = ?
            WHERE
                id = ?
            "###,
            self.task_id,
            self.version_hash,
            content_manifest_json, // Use the let binding
            self.is_live,
            self.created_at,
            self.id
        )
        .execute(executor)
        .await?;
        Ok(())
    }

    async fn delete<'e, E>(executor: E, id: i64) -> Result<(), Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        sqlx::query!("DELETE FROM statement_versions WHERE id = ?", id)
            .execute(executor)
            .await?;
        Ok(())
    }

    async fn get_by_id<'e, E>(executor: E, id: i64) -> Result<Option<Self>, Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let statement_version = sqlx::query_as!(
            StatementVersion,
            r###" 
            SELECT
                id,
                task_id,
                version_hash,
                content_manifest as "content_manifest: Json<std::collections::HashMap<String, String>>",
                is_live,
                created_at
            FROM
                statement_versions
            WHERE
                id = ?
            "###,
            id
        )
        .fetch_optional(executor)
        .await?;
        Ok(statement_version)
    }

    async fn get_all<'e, E>(executor: E) -> Result<Vec<Self>, Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let statement_versions = sqlx::query_as!(
            StatementVersion,
            r###" 
            SELECT
                id,
                task_id,
                version_hash,
                content_manifest as "content_manifest: Json<std::collections::HashMap<String, String>>",
                is_live,
                created_at
            FROM
                statement_versions
            ORDER BY
                id DESC
            "###
        )
        .fetch_all(executor)
        .await?;
        Ok(statement_versions)
    }
}
