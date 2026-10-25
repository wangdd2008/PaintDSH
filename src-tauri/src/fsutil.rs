//! 文件系统小工具：原子写入。
//!
//! 所有落盘写入都先写 `*.tmp` 再 `rename`，避免崩溃时留下半截文件，
//! 也避免 DSH 的文件监听读到不完整内容（engineering-contract 第 11 节）。

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::AppError;

/// 原子写入：同目录临时文件 → fsync → rename。
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            AppError::io(format!("创建目录 {} 失败：{error}", parent.display()))
        })?;
    }

    let temp_path = temp_path_for(path);
    {
        let mut file = std::fs::File::create(&temp_path).map_err(|error| {
            AppError::io(format!("创建临时文件 {} 失败：{error}", temp_path.display()))
        })?;
        file.write_all(bytes)
            .map_err(|error| AppError::io(format!("写入 {} 失败：{error}", temp_path.display())))?;
        file.sync_all()
            .map_err(|error| AppError::io(format!("同步 {} 失败：{error}", temp_path.display())))?;
    }

    std::fs::rename(&temp_path, path).map_err(|error| {
        let _ = std::fs::remove_file(&temp_path);
        AppError::io(format!(
            "替换 {} 失败：{error}",
            path.display()
        ))
    })?;

    Ok(())
}

/// 内容相同就跳过写入（幂等同步用）。
pub fn write_if_changed(path: &Path, bytes: &[u8]) -> Result<bool, AppError> {
    if let Ok(existing) = std::fs::read(path) {
        if existing == bytes {
            return Ok(false);
        }
    }
    write_atomic(path, bytes)?;
    Ok(true)
}

/// 删除文件或目录（不存在时视为成功）。
pub fn remove_path(path: &Path) -> Result<(), AppError> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        std::fs::remove_dir_all(path)
            .map_err(|error| AppError::io(format!("删除 {} 失败：{error}", path.display())))
    } else {
        std::fs::remove_file(path)
            .map_err(|error| AppError::io(format!("删除 {} 失败：{error}", path.display())))
    }
}

fn temp_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "paint".to_string());
    path.with_file_name(format!("{file_name}.{}.tmp", uuid::Uuid::new_v4()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("paint-fsutil-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("创建临时目录");
        dir
    }

    #[test]
    fn write_atomic_creates_parent_dirs_and_leaves_no_temp_file() {
        let dir = temp_dir();
        let path = dir.join("nested").join("deep").join("file.txt");

        write_atomic(&path, b"hello").expect("写入");

        assert_eq!(std::fs::read(&path).expect("读取"), b"hello");
        let leftovers: Vec<_> = std::fs::read_dir(path.parent().unwrap())
            .expect("列目录")
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "不应残留临时文件");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_if_changed_skips_identical_content() {
        let dir = temp_dir();
        let path = dir.join("same.txt");

        assert!(write_if_changed(&path, b"v1").expect("首次写入"));
        assert!(!write_if_changed(&path, b"v1").expect("内容相同"));
        assert!(write_if_changed(&path, b"v2").expect("内容变化"));
        assert_eq!(std::fs::read(&path).expect("读取"), b"v2");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn remove_path_tolerates_missing_targets() {
        let dir = temp_dir();
        let path = dir.join("gone.txt");

        remove_path(&path).expect("不存在也应成功");
        std::fs::write(&path, b"x").expect("写文件");
        remove_path(&path).expect("删除文件");
        assert!(!path.exists());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
