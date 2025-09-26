use gloo_storage::errors::StorageError;
use gloo_storage::{SessionStorage, Storage};
use uuid::Uuid;

const SESSION_TOKEN_KEY: &str = "session_token";

pub fn get_session_token() -> String {
    match SessionStorage::get(SESSION_TOKEN_KEY) {
        Ok(val) => val,
        Err(StorageError::KeyNotFound(_)) | Err(StorageError::SerdeError(_)) => {
            let token = Uuid::new_v4().to_string();
            SessionStorage::set(SESSION_TOKEN_KEY, &token).unwrap();
            token
        }
        Err(e) => panic!("Failed to access session storage: {}", e),
    }
}
