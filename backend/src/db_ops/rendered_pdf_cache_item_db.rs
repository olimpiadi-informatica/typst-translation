use common::error::Error;
use common::rendered_pdf_cache_item::RenderedPdfCacheItem;
use sqlx::{Executor, Sqlite};

use crate::db_ops::DatabaseOps;

impl DatabaseOps for RenderedPdfCacheItem {
    async fn insert<'e, E>(&mut self, executor: E) -> Result<(), Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let row = sqlx::query!(
            r###" 
            INSERT INTO rendered_pdf_cache (
                statement_version_id,
                language_id,
                pdf_hash
            )
            VALUES (?, ?, ?)
            RETURNING id
            "###,
            self.statement_version_id,
            self.language_id,
            self.pdf_hash,
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
            UPDATE rendered_pdf_cache
            SET
                statement_version_id = ?,
                language_id = ?,
                pdf_hash = ?
            WHERE
                id = ?
            "###,
            self.statement_version_id,
            self.language_id,
            self.pdf_hash,
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
        sqlx::query!("DELETE FROM rendered_pdf_cache WHERE id = ?", id)
            .execute(executor)
            .await?;
        Ok(())
    }

    async fn get_by_id<'e, E>(executor: E, id: i64) -> Result<Option<Self>, Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let rendered_pdf_cache_item = sqlx::query_as!(
            RenderedPdfCacheItem,
            r###" 
            SELECT
                id,
                statement_version_id,
                language_id,
                pdf_hash
            FROM
                rendered_pdf_cache
            WHERE
                id = ?
            "###,
            id
        )
        .fetch_optional(executor)
        .await?;
        Ok(rendered_pdf_cache_item)
    }

    async fn get_all<'e, E>(executor: E) -> Result<Vec<Self>, Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let rendered_pdf_cache_items = sqlx::query_as!(
            RenderedPdfCacheItem,
            r###" 
            SELECT
                id,
                statement_version_id,
                language_id,
                pdf_hash
            FROM
                rendered_pdf_cache
            ORDER BY
                id DESC
            "###
        )
        .fetch_all(executor)
        .await?;
        Ok(rendered_pdf_cache_items)
    }
}
