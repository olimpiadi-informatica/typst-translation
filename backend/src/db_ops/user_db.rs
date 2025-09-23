use common::error::Error;
use common::user::User;
use sqlx::{Executor, Sqlite};

pub async fn get_user_by_username<'e, E>(executor: E, username: &str) -> Result<Option<User>, Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    let user = sqlx::query_as!(
        User,
        r###"
        SELECT
            id,
            username,
            password,
            login_epoch,
            automatic_translation_budget,
            tokens_used,
            name
        FROM
            users
        WHERE
            username = ?
        "###,
        username
    )
    .fetch_optional(executor)
    .await?;
    Ok(user)
}

pub async fn insert<'e, E>(user: &mut User, executor: E) -> Result<(), Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    let row = sqlx::query!(
        r###"
        INSERT INTO users (
            username,
            password,
            login_epoch,
            automatic_translation_budget,
            tokens_used,
            name
        )
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING id
        "###,
        user.username,
        user.password,
        user.login_epoch,
        user.automatic_translation_budget,
        user.tokens_used,
        user.name,
    )
    .fetch_one(executor)
    .await?;
    user.id = row.id;
    Ok(())
}

pub async fn update<'e, E>(user: &User, executor: E) -> Result<(), Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        r###"
        UPDATE users
        SET
            username = ?,
            password = ?,
            login_epoch = ?,
            automatic_translation_budget = ?,
            tokens_used = ?,
            name = ?
        WHERE
            id = ?
        "###,
        user.username,
        user.password,
        user.login_epoch,
        user.automatic_translation_budget,
        user.tokens_used,
        user.name,
        user.id
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn delete<'e, E>(executor: E, id: i64) -> Result<(), Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!("DELETE FROM users WHERE id = ?", id)
        .execute(executor)
        .await?;
    Ok(())
}

pub async fn get_by_id<'e, E>(executor: E, id: i64) -> Result<Option<User>, Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    let user = sqlx::query_as!(
        User,
        r###"
        SELECT
            id,
            username,
            password,
            login_epoch,
            automatic_translation_budget,
            tokens_used,
            name
        FROM
            users
        WHERE
            id = ?
        "###,
        id
    )
    .fetch_optional(executor)
    .await?;
    Ok(user)
}

pub async fn get_all<'e, E>(executor: E) -> Result<Vec<User>, Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    let users = sqlx::query_as!(
        User,
        r###"
        SELECT
            id,
            username,
            password,
            login_epoch,
            automatic_translation_budget,
            tokens_used,
            name
        FROM
            users
        ORDER BY
            id DESC
        "###,
    )
    .fetch_all(executor)
    .await?;
    Ok(users)
}
