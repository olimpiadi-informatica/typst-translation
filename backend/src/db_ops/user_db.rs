use common::error::Error;
use common::user::User;
use sqlx::{Executor, Sqlite, SqlitePool};

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

pub async fn set_skip_envelope_verification(
    pool: &SqlitePool,
    user_id: i64,
    contest_id: i64,
    skip: bool,
) -> Result<(), Error> {
    let mut tx = pool.begin().await?;

    let result = sqlx::query!(
        r#"
            UPDATE user_contest_status
            SET skip_envelope_verification = ?
            WHERE user_id = ? AND contest_id = ? AND finalized_translations = FALSE
        "#,
        skip,
        user_id,
        contest_id
    )
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(Error::InvalidInput(
            "Cannot change skip_envelope_verification for a finalized translation.".to_string(),
        ));
    }

    tx.commit().await?;
    Ok(())
}

pub async fn set_automatic_translation_budget<'e, E>(
    executor: E,
    user_id: i64,
    new_budget: i64,
) -> Result<(), Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        "UPDATE users SET automatic_translation_budget = ? WHERE id = ?",
        new_budget,
        user_id
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn set_all_automatic_translation_budgets<'e, E>(
    executor: E,
    new_budget: i64,
) -> Result<(), Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        "UPDATE users SET automatic_translation_budget = ?",
        new_budget
    )
    .execute(executor)
    .await?;
    Ok(())
}
