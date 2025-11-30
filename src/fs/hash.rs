use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use sha2::{Sha256, Digest};
use md5;
use sha1::Sha1;

pub async fn compute_hash(path: &Path, hash_type: HashType) -> Result<String, crate::fs::error::FileOperationError> {
    let mut file = File::open(path).await
        .map_err(|e| crate::fs::error::FileOperationError::Io(e))?;
    
    let mut buffer = vec![0u8; 8192];
    
    match hash_type {
        HashType::MD5 => {
            let mut hasher = md5::Context::new();
            loop {
                let n = file.read(&mut buffer).await
                    .map_err(|e| crate::fs::error::FileOperationError::Io(e))?;
                if n == 0 {
                    break;
                }
                hasher.consume(&buffer[..n]);
            }
            let result = hasher.compute();
            Ok(format!("{:x}", result))
        }
        HashType::SHA1 => {
            let mut hasher = Sha1::new();
            loop {
                let n = file.read(&mut buffer).await
                    .map_err(|e| crate::fs::error::FileOperationError::Io(e))?;
                if n == 0 {
                    break;
                }
                hasher.update(&buffer[..n]);
            }
            let result = hasher.finalize();
            Ok(result.iter().map(|b| format!("{:02x}", b)).collect::<String>())
        }
        HashType::SHA256 => {
            let mut hasher = Sha256::new();
            loop {
                let n = file.read(&mut buffer).await
                    .map_err(|e| crate::fs::error::FileOperationError::Io(e))?;
                if n == 0 {
                    break;
                }
                hasher.update(&buffer[..n]);
            }
            let result = hasher.finalize();
            Ok(result.iter().map(|b| format!("{:02x}", b)).collect::<String>())
        }
    }
}

#[derive(Clone, Copy)]
pub enum HashType {
    MD5,
    SHA1,
    SHA256,
}

