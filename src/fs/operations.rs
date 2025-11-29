use std::path::{Path, PathBuf};
use std::io;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send + Sync>;

pub async fn copy_file(
    src: &Path,
    dst: &Path,
    progress: Option<ProgressCallback>,
) -> Result<(), io::Error> {
    let metadata = fs::metadata(src).await?;
    let total_size = metadata.len();

    let mut src_file = fs::File::open(src).await?;
    let mut dst_file = fs::File::create(dst).await?;

    let mut buffer = vec![0u8; 8192];
    let mut copied = 0u64;

    loop {
        let bytes_read = src_file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }

        dst_file.write_all(&buffer[..bytes_read]).await?;
        copied += bytes_read as u64;

        if let Some(ref callback) = progress {
            callback(copied, total_size);
        }
    }

    dst_file.sync_all().await?;
    Ok(())
}

pub async fn move_file(
    src: &Path,
    dst: &Path,
    progress: Option<ProgressCallback>,
) -> Result<(), io::Error> {
    copy_file(src, dst, progress).await?;
    fs::remove_file(src).await?;
    Ok(())
}

pub async fn copy_directory(
    src: &Path,
    dst: &Path,
    progress: Option<ProgressCallback>,
) -> Result<(), io::Error> {
    copy_directory_internal(src, dst, progress).await
}

fn copy_directory_internal(
    src: &Path,
    dst: &Path,
    progress: Option<ProgressCallback>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), io::Error>> + Send>> {
    let src = src.to_path_buf();
    let dst = dst.to_path_buf();
    Box::pin(async move {
        fs::create_dir_all(&dst).await?;

        let mut entries = fs::read_dir(&src).await?;
        let mut total_size = 0u64;
        let mut processed = 0u64;

        let mut files = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                let dst_subdir = dst.join(path.file_name().unwrap());
                copy_directory_internal(&path, &dst_subdir, None).await?;
        } else {
            let metadata = entry.metadata().await?;
            total_size += metadata.len();
            files.push(path);
        }
    }

    for file in files {
        let dst_file = dst.join(file.file_name().unwrap());
        copy_file(&file, &dst_file, None).await?;
        processed += fs::metadata(&file).await?.len();
        if let Some(ref callback) = progress {
            callback(processed, total_size);
        }
    }

        Ok(())
    })
}

pub async fn move_directory(
    src: &Path,
    dst: &Path,
    progress: Option<ProgressCallback>,
) -> Result<(), io::Error> {
    copy_directory(src, dst, progress).await?;
    fs::remove_dir_all(src).await?;
    Ok(())
}

pub async fn delete_file(path: &Path) -> Result<(), io::Error> {
    fs::remove_file(path).await
}

pub async fn delete_directory(path: &Path) -> Result<(), io::Error> {
    fs::remove_dir_all(path).await
}

pub async fn rename_path(src: &Path, new_name: &str) -> Result<PathBuf, io::Error> {
    let parent = src.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "Path has no parent")
    })?;
    let new_path = parent.join(new_name);
    fs::rename(src, &new_path).await?;
    Ok(new_path)
}

