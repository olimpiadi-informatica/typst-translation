use common::error::Error;
use sqlx::{Executor, Sqlite};

mod user_db;
mod language_db;
mod contest_db;
mod task_db;
mod statement_version_db;
mod contestant_db;
mod translation_db;
mod user_contest_status_db;
mod printed_document_db;
mod draft_print_queue_item_db;
mod rendered_pdf_cache_item_db;

pub use user_db::*;

#[allow(async_fn_in_trait)]
pub trait DatabaseOps: Sized {
    /// Ignores the id field and sets it in `self` to the actual value from the db.
    async fn insert<'e, E>(&mut self, executor: E) -> Result<(), Error>
    where
        E: Executor<'e, Database = Sqlite>;

    /// Update the row with the given `id` to the given values.
    async fn update<'e, E>(&self, executor: E) -> Result<(), Error>
    where
        E: Executor<'e, Database = Sqlite>;

    /// Removes the entity with the given id.
    async fn delete<'e, E>(executor: E, id: i64) -> Result<(), Error>
    where
        E: Executor<'e, Database = Sqlite>;

    /// Fetches the entity with the given id.
    async fn get_by_id<'e, E>(executor: E, id: i64) -> Result<Option<Self>, Error>
    where
        E: Executor<'e, Database = Sqlite>;

    /// Fetches all the entities of this type, in a reasonable (entity-defined) order.
    async fn get_all<'e, E>(executor: E) -> Result<Vec<Self>, Error>
    where
        E: Executor<'e, Database = Sqlite>;
}