//! Periodically reads the branch and recent commits from a Git repository.

use super::data::GitCommit;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

const REFRESH_INTERVAL: Duration = Duration::from_secs(15);
const COMMIT_COUNT: usize = 5;

#[derive(Debug, Clone, Default)]
pub struct GitSnapshot {
    pub branch: Option<String>,
    pub commits: Vec<GitCommit>,
}

pub struct GitSensor {
    repository: PathBuf,
    last_refresh: Option<Instant>,
    snapshot: GitSnapshot,
}

impl GitSensor {
    pub fn new(repository: impl Into<PathBuf>) -> Self {
        Self {
            repository: repository.into(),
            last_refresh: None,
            snapshot: GitSnapshot::default(),
        }
    }

    pub fn set_repository(&mut self, repository: impl Into<PathBuf>) {
        self.repository = repository.into();
        self.last_refresh = None;
        self.snapshot = GitSnapshot::default();
    }

    pub fn sample(&mut self) -> GitSnapshot {
        if self
            .last_refresh
            .is_none_or(|last| last.elapsed() >= REFRESH_INTERVAL)
        {
            self.snapshot = self.read_repository();
            self.last_refresh = Some(Instant::now());
        }
        self.snapshot.clone()
    }

    fn read_repository(&self) -> GitSnapshot {
        let branch = Command::new("git")
            .args(["-C"])
            .arg(&self.repository)
            .args(["branch", "--show-current"])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .filter(|branch| !branch.is_empty());

        let commits = Command::new("git")
            .args(["-C"])
            .arg(&self.repository)
            .args(["log", &format!("-{}", COMMIT_COUNT), "--format=%h%x1f%s"])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter_map(|line| {
                        let (hash, subject) = line.split_once('\x1f')?;
                        Some(GitCommit {
                            hash: hash.to_string(),
                            subject: subject.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        GitSnapshot { branch, commits }
    }
}
