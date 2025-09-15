use std::path::PathBuf;

use color_eyre::eyre::{Result, bail};

pub const DOCUMENTS_DIR: &str = "documents";
pub const FILES_DIR: &str = "files";

pub fn hash(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

pub fn path_of_file(base_dir: &str, hash: &str) -> Result<PathBuf> {
    if !hash.chars().all(|x| x.is_ascii_hexdigit()) || hash.len() != blake3::OUT_LEN * 2 {
        bail!("invalid hash")
    }
    Ok(PathBuf::new()
        .join(base_dir)
        .join(&hash[0..2])
        .join(&hash[2..4])
        .join(hash))
}

pub async fn save_file(base_dir: &str, data: &[u8]) -> Result<String, tokio::io::Error> {
    let hash = hash(data);
    let save_path = path_of_file(base_dir, &hash).map_err(tokio::io::Error::other)?;
    tokio::fs::create_dir_all(&save_path.parent().unwrap()).await?;
    tokio::fs::write(save_path, data).await?;
    Ok(hash.to_string())
}

/*
pub async fn get_file(
    base_dir: &str,
    hash: &str,
    filename: &str,
    user: Option<UserExtractor>,
    required_permission: UserPermission,
) -> Result<Attachment<Vec<u8>>, Error> {
    info!(?user, ?filename, ?hash, "File access attempt");
    let path = path_of_file(base_dir, hash).map_err(|x| {
        warn!("requested invalid hash: {hash} err: {x}");
        Error::NotFound
    })?;
    if !has_permission(user.as_deref(), required_permission) {
        return Err(Error::Forbidden);
    }
    let file_contents = tokio::fs::read(path).await.map_err(|x| {
        warn!("Error retrieving file with hash {hash}: {x}");
        Error::NotFound
    })?;
    Ok(Attachment::new(file_contents)
        .filename(filename)
        .content_type("application/octect-stream"))
}
*/
