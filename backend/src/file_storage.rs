use std::path::PathBuf;

use axum::extract::Path;
use axum_extra::response::Attachment;
use color_eyre::eyre::{Result, bail};
use common::error::Error;
use tracing::{info, warn};

use crate::auth::AuthUser;

pub const FILES_DIR: &str = "files";

pub fn hash(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

pub fn path_of_file(hash: &str) -> Result<PathBuf> {
    if !hash.chars().all(|x| x.is_ascii_hexdigit()) || hash.len() != blake3::OUT_LEN * 2 {
        bail!("invalid hash")
    }
    Ok(PathBuf::new()
        .join(FILES_DIR)
        .join(&hash[0..2])
        .join(&hash[2..4])
        .join(hash))
}

pub async fn save_file(data: &[u8]) -> Result<String, tokio::io::Error> {
    let hash = hash(data);
    let save_path = path_of_file(&hash).map_err(tokio::io::Error::other)?;
    tokio::fs::create_dir_all(&save_path.parent().unwrap()).await?;
    let tempdir = tempfile::tempdir_in(FILES_DIR)?;
    let tempfile = tempdir.path().join(&hash);
    tokio::fs::write(&tempfile, data).await?;
    tokio::fs::rename(&tempfile, save_path).await?;
    Ok(hash.to_string())
}

pub async fn get_file(
    Path((hash, filename)): Path<(String, String)>,
    user: AuthUser,
) -> Result<Attachment<Vec<u8>>, Error> {
    info!(?user, ?filename, ?hash, "File access attempt");
    let path = path_of_file(&hash).map_err(|x| {
        warn!("requested invalid hash: {hash} err: {x}");
        Error::NotFound
    })?;
    let file_contents = tokio::fs::read(path).await.map_err(|x| {
        warn!("Error retrieving file with hash {hash}: {x}");
        Error::NotFound
    })?;
    Ok(Attachment::new(file_contents)
        .filename(filename)
        .content_type("application/octect-stream"))
}
