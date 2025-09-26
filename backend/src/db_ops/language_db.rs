use common::error::Error;
use common::language::Language;
use sqlx::{Executor, Pool, Sqlite};

pub async fn insert<'e, E>(language: &mut Language, executor: E) -> Result<(), Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    let row = sqlx::query!(
        r###"
        INSERT INTO languages (
            code,
            user_id,
            public
        )
        VALUES (?, ?, ?)
        RETURNING id
        "###,
        language.code,
        language.user_id,
        language.public,
    )
    .fetch_one(executor)
    .await?;
    language.id = row.id;
    Ok(())
}

pub async fn update<'e, E>(language: &Language, executor: E) -> Result<(), Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        r###"
        UPDATE languages
        SET
            code = ?,
            user_id = ?,
            public = ?
        WHERE
            id = ?
        "###,
        language.code,
        language.user_id,
        language.public,
        language.id
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn delete<'e, E>(executor: E, id: i64) -> Result<(), Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!("DELETE FROM languages WHERE id = ?", id)
        .execute(executor)
        .await?;
    Ok(())
}

pub async fn get_by_id<'e, E>(executor: E, id: i64) -> Result<Language, Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    let language = sqlx::query_as!(Language, "SELECT * FROM languages WHERE id = ?", id)
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NotFound)?;
    Ok(language)
}

pub async fn get_all<'e, E>(executor: E) -> Result<Vec<Language>, Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    let languages = sqlx::query_as!(
        Language,
        r###"
        SELECT
            id,
            code,
            user_id,
            public
        FROM
            languages
        ORDER BY
            id DESC
        "###
    )
    .fetch_all(executor)
    .await?;
    Ok(languages)
}

pub async fn get_available_languages<'e, E>(
    executor: E,
    user_id: i64,
) -> Result<Vec<Language>, Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    let languages = sqlx::query_as!(
        Language,
        r###"
        SELECT
            id,
            code,
            user_id,
            public
        FROM
            languages
        WHERE
            public = TRUE OR user_id = ?
        ORDER BY
            id DESC
        "###,
        user_id
    )
    .fetch_all(executor)
    .await?;
    Ok(languages)
}

pub async fn get_user_owned_languages<'e, E>(
    executor: E,
    user_id: i64,
) -> Result<Vec<Language>, Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    let languages = sqlx::query_as!(
        Language,
        r###"
        SELECT
            id,
            code,
            user_id,
            public
        FROM
            languages
        WHERE
            user_id = ?
        ORDER BY
            id DESC
        "###,
        user_id
    )
    .fetch_all(executor)
    .await?;
    Ok(languages)
}

pub async fn toggle_language_public_status(
    pool: &Pool<Sqlite>,
    language_id: i64,
    new_status: bool,
) -> Result<(), Error> {
    use crate::db_ops::contestant_db;

    let mut tx = pool.begin().await?;

    let language = get_by_id(&mut *tx, language_id).await?;

    if !new_status {
        // If we are trying to make the language private, check if any contestants are assigned to it.
        let assigned_contestants =
            contestant_db::get_contestants_by_language_id(&mut *tx, language_id).await?;
        //if !assigned_contestants.is_empty() {
        if assigned_contestants
            .into_iter()
            .any(|c| c.user_id != language.user_id)
        {
            return Err(Error::InvalidInput(
                "Cannot make language private while contestants are assigned to it.".to_string(),
            ));
        }
    }

    sqlx::query!(
        r###"
        UPDATE languages
        SET
            public = ?
        WHERE
            id = ?
        "###,
        new_status,
        language_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
