use common::error::Error;
use common::printed_document::PrintedDocument;
use sqlx::{Executor, Sqlite};

use crate::db_ops::DatabaseOps;

impl DatabaseOps for PrintedDocument {
    async fn insert<'e, E>(&mut self, executor: E) -> Result<(), Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let row = sqlx::query!(
            r###" 
            INSERT INTO printed_documents (
                contestant_id,
                statement_version_id,
                language_id,
                printed_at
            )
            VALUES (?, ?, ?, ?)
            RETURNING id
            "###,
            self.contestant_id,
            self.statement_version_id,
            self.language_id,
            self.printed_at,
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
        sqlx::query!(
            r###" 
            UPDATE printed_documents
            SET
                contestant_id = ?,
                statement_version_id = ?,
                language_id = ?,
                printed_at = ?
            WHERE
                id = ?
            "###,
            self.contestant_id,
            self.statement_version_id,
            self.language_id,
            self.printed_at,
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
        sqlx::query!("DELETE FROM printed_documents WHERE id = ?", id)
            .execute(executor)
            .await?;
        Ok(())
    }

    async fn get_by_id<'e, E>(executor: E, id: i64) -> Result<Option<Self>, Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let printed_document = sqlx::query_as!(
            PrintedDocument,
            r###" 
            SELECT
                id,
                contestant_id,
                statement_version_id,
                language_id,
                printed_at
            FROM
                printed_documents
            WHERE
                id = ?
            "###,
            id
        )
        .fetch_optional(executor)
        .await?;
        Ok(printed_document)
    }

    async fn get_all<'e, E>(executor: E) -> Result<Vec<Self>, Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let printed_documents = sqlx::query_as!(
            PrintedDocument,
            r###" 
            SELECT
                id,
                contestant_id,
                statement_version_id,
                language_id,
                printed_at
            FROM
                printed_documents
            ORDER BY
                id DESC
            "###
        )
        .fetch_all(executor)
        .await?;
        Ok(printed_documents)
    }
}