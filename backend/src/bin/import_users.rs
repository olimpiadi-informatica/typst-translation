use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::Result;
use serde::Deserialize;
use serde_jsonlines::json_lines;
use sqlx::query;

#[derive(Parser)]
struct Args {
    /// Import users from a jsonline file.
    #[clap(long)]
    users: PathBuf,
    /// Not supported yet.
    #[clap(long, default_value_t = false)]
    update: bool,
}

#[derive(Debug, Deserialize)]
struct User {
    contestants: Vec<Contestant>,
    username: String,
    languages: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Contestant {
    name: String,
    code: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Args::parse();
    let db = backend::init().await?;

    // TODO(veluca): make this into actual web endpoints for admins to use.

    let users = json_lines(cli.users)?.collect::<std::io::Result<Vec<User>>>()?;

    let mut tx = db.begin().await?;

    for user in users {
        let user_id = query!(
            "INSERT INTO users(username, password, login_epoch) VALUES (?, 'satisfy-mollusk-encrypt', 0) RETURNING id;",
            user.username
       )
            .fetch_one(&mut *tx)
            .await?
            .id;

        for lang in user.languages {
            let _lang_id = query!(
                "INSERT INTO languages(code, user_id) VALUES (?, ?) RETURNING id;",
                lang,
                user_id
            )
            .fetch_one(&mut *tx)
            .await?
            .id;
        }

        for contestant in user.contestants {
            let _contestant_id = query!(
                "INSERT INTO contestants(code, name, online_bit, user_id) VALUES (?, ?, 0, ?) RETURNING id;",
                contestant.code,
                contestant.name,
                user_id
            )
            .fetch_one(&mut *tx)
            .await?
            .id;
        }
    }

    tx.commit().await?;

    Ok(())
}
