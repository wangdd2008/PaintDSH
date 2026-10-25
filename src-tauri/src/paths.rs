//! PAINT 数据目录解析（`~/.paint`，可用 `PAINT_HOME` 覆盖）。
//!
//! 业务逻辑禁止直接读环境变量或用户目录——统一走 [`PaintHome`]，测试注入临时目录即可
//! （engineering-contract 第 8 节）。

use std::path::{Path, PathBuf};

use crate::error::{AppError, ErrorKind};

/// 覆盖根目录的环境变量名（测试与便携模式用）。
pub const PAINT_HOME_ENV: &str = "PAINT_HOME";

/// SQLite 数据库文件名。
pub const DB_FILE_NAME: &str = "paint.db";

/// PAINT 的数据根目录及其标准子目录。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaintHome {
    root: PathBuf,
}

impl PaintHome {
    /// 以给定路径为根（不校验存在性）。
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// 生产入口：`PAINT_HOME` 优先，否则 `~/.paint`。
    pub fn resolve() -> Result<Self, AppError> {
        Self::resolve_with(|key| std::env::var(key).ok())
    }

    /// 可注入的环境读取：测试传闭包，避免改进程环境导致的并发问题。
    pub fn resolve_with<F>(lookup: F) -> Result<Self, AppError>
    where
        F: Fn(&str) -> Option<String>,
    {
        if let Some(value) = lookup(PAINT_HOME_ENV) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Ok(Self::new(trimmed));
            }
        }

        let home = dirs::home_dir().ok_or_else(|| {
            AppError::new(
                ErrorKind::Io,
                "无法解析用户主目录，请通过 PAINT_HOME 指定数据目录",
            )
        })?;
        Ok(Self::new(home.join(".paint")))
    }

    /// 根目录。
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// PAINT 技能主目录（`<root>/skills`），同步到 DSH 技能目录的源头。
    pub fn skills_dir(&self) -> PathBuf {
        self.root.join("skills")
    }

    /// 知识库目录（`<root>/knowledge`），会注册为 DSH 工作区。
    pub fn knowledge_dir(&self) -> PathBuf {
        self.root.join("knowledge")
    }

    /// DSH_HOME（`<root>/harness`）。
    pub fn harness_home(&self) -> PathBuf {
        self.root.join("harness")
    }

    /// SQLite 数据库路径（`<root>/paint.db`）。
    pub fn db_path(&self) -> PathBuf {
        self.root.join(DB_FILE_NAME)
    }

    /// 创建根目录与标准子目录（幂等）。
    pub fn ensure_dirs(&self) -> Result<(), AppError> {
        for dir in [
            self.root.clone(),
            self.skills_dir(),
            self.knowledge_dir(),
            self.harness_home(),
        ] {
            std::fs::create_dir_all(&dir).map_err(|error| {
                AppError::new(
                    ErrorKind::Io,
                    format!("创建目录 {} 失败：{error}", dir.display()),
                )
            })?;
        }
        Ok(())
    }
}

/// 生产环境的便捷入口；需要注入时用 [`PaintHome::resolve_with`]。
pub fn paint_home() -> Result<PaintHome, AppError> {
    PaintHome::resolve()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root() -> PathBuf {
        std::env::temp_dir().join(format!("paint-home-test-{}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn env_override_wins() {
        let root = temp_root();
        let home = PaintHome::resolve_with(|key| {
            (key == PAINT_HOME_ENV).then(|| root.to_string_lossy().into_owned())
        })
        .expect("解析 PaintHome");

        assert_eq!(home.root(), root.as_path());
    }

    #[test]
    fn blank_env_value_falls_back_to_user_home() {
        let home = PaintHome::resolve_with(|_| Some("   ".to_string())).expect("解析 PaintHome");

        assert!(home.root().ends_with(".paint"), "root = {:?}", home.root());
    }

    #[test]
    fn standard_paths_are_under_root() {
        let home = PaintHome::new("C:/tmp/paint-home");

        assert_eq!(home.skills_dir(), PathBuf::from("C:/tmp/paint-home/skills"));
        assert_eq!(
            home.knowledge_dir(),
            PathBuf::from("C:/tmp/paint-home/knowledge")
        );
        assert_eq!(
            home.harness_home(),
            PathBuf::from("C:/tmp/paint-home/harness")
        );
        assert_eq!(
            home.db_path(),
            PathBuf::from("C:/tmp/paint-home/paint.db")
        );
    }

    #[test]
    fn ensure_dirs_creates_every_standard_directory() {
        let root = temp_root();
        let home = PaintHome::new(&root);

        home.ensure_dirs().expect("创建目录");
        assert!(home.skills_dir().is_dir());
        assert!(home.knowledge_dir().is_dir());
        assert!(home.harness_home().is_dir());

        let _ = std::fs::remove_dir_all(&root);
    }
}
