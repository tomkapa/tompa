// git_manager.rs — Git Manager Actor (T21)
//
// Handles all git operations for the agent: story branch creation, worktree
// management (create/remove), commit-and-push after task completion.
//
// Uses gix for repo validation and branch creation; git subprocess for
// worktree management, staging, commit, and push (gix worktree-add and push
// are not yet stable).

use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

use anyhow::{Context, Result, anyhow, bail};
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;

use shared::types::TaskContext;

use crate::dispatcher::DispatchMessage;

// ── Public message type ───────────────────────────────────────────────────────

#[derive(Debug)]
pub enum GitMessage {
    /// Ensure a story branch and worktree exist before task execution starts.
    EnsureWorktree {
        story_id: Uuid,
        task_id: Uuid,
        session_id: String,
        context: TaskContext,
    },
    /// Stage, commit, and push all changes in a worktree after task completion.
    CommitAndPush {
        story_id: Uuid,
        task_id: Uuid,
        worktree: PathBuf,
    },
    /// Remove the worktree when a story is fully done.
    #[allow(dead_code)]
    RemoveWorktree { story_id: Uuid },
}

// ── Actor ─────────────────────────────────────────────────────────────────────

pub struct GitManager {
    repo_path: PathBuf,
    github_access_token: Option<String>,
    dispatch_tx: mpsc::Sender<DispatchMessage>,
    rx: mpsc::Receiver<GitMessage>,
}

impl GitManager {
    pub fn new(
        github_repo_url: Option<String>,
        github_access_token: Option<String>,
        dispatch_tx: mpsc::Sender<DispatchMessage>,
        rx: mpsc::Receiver<GitMessage>,
    ) -> Self {
        // In the container the repo is always checked out at CWD.
        // The URL is stored for push auth but the local path is always ".".
        let _ = github_repo_url; // kept for future use (clone path resolution)
        let repo_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            repo_path,
            github_access_token,
            dispatch_tx,
            rx,
        }
    }

    /// Constructor for tests — allows an explicit repo path.
    #[cfg(test)]
    pub fn new_with_path(
        repo_path: PathBuf,
        github_access_token: Option<String>,
        dispatch_tx: mpsc::Sender<DispatchMessage>,
        rx: mpsc::Receiver<GitMessage>,
    ) -> Self {
        Self {
            repo_path,
            github_access_token,
            dispatch_tx,
            rx,
        }
    }

    pub async fn run(mut self) {
        info!(repo = ?self.repo_path, "git_manager starting");
        while let Some(msg) = self.rx.recv().await {
            self.handle(msg).await;
        }
        info!("git_manager shutting down");
    }

    async fn handle(&self, msg: GitMessage) {
        match msg {
            GitMessage::EnsureWorktree {
                story_id,
                task_id,
                session_id,
                context,
            } => match self.ensure_worktree(story_id) {
                Ok(worktree) => {
                    self.send(DispatchMessage::WorktreeReady {
                        story_id,
                        task_id,
                        session_id,
                        worktree,
                        context,
                    })
                    .await;
                }
                Err(e) => {
                    error!(%story_id, %task_id, %e, "ensure_worktree failed");
                    self.send(DispatchMessage::WorktreeFailed {
                        task_id,
                        error: e.to_string(),
                    })
                    .await;
                }
            },
            GitMessage::CommitAndPush {
                story_id,
                task_id,
                worktree,
            } => {
                let branch = story_branch_name(story_id);
                match self.commit_and_push(&worktree, task_id, &branch) {
                    Ok(commit_sha) => {
                        self.send(DispatchMessage::CommitComplete {
                            task_id,
                            commit_sha,
                        })
                        .await;
                    }
                    Err(e) => {
                        error!(%task_id, %e, "commit_and_push failed");
                        self.send(DispatchMessage::CommitFailed {
                            task_id,
                            error: e.to_string(),
                        })
                        .await;
                    }
                }
            }
            GitMessage::RemoveWorktree { story_id } => {
                let path = self.worktree_path(story_id);
                if let Err(e) = self.remove_worktree(&path) {
                    warn!(%story_id, %e, "remove_worktree failed (non-fatal)");
                }
            }
        }
    }

    // ── Git operations ────────────────────────────────────────────────────────

    /// Ensure the story branch exists and the worktree is checked out.
    fn ensure_worktree(&self, story_id: Uuid) -> Result<PathBuf> {
        let branch = story_branch_name(story_id);
        let worktree_path = self.worktree_path(story_id);

        self.ensure_branch(&branch)?;

        if !worktree_path.exists() {
            self.create_worktree(&worktree_path, &branch)?;
        }

        Ok(worktree_path)
    }

    /// Create the story branch from HEAD using gix. No-op if it already exists.
    fn ensure_branch(&self, branch: &str) -> Result<()> {
        let repo = gix::open(&self.repo_path)
            .with_context(|| format!("open repo {:?}", self.repo_path))?;

        let full_ref = format!("refs/heads/{branch}");

        if repo.find_reference(full_ref.as_str()).is_ok() {
            info!(%branch, "branch already exists");
            return Ok(());
        }

        // Resolve HEAD to a concrete commit id.
        let commit_id = repo.head_commit().context("resolve HEAD commit")?.id;

        repo.reference(
            full_ref.as_str(),
            commit_id,
            gix::refs::transaction::PreviousValue::MustNotExist,
            format!("T21: create story branch {branch}").as_str(),
        )
        .with_context(|| format!("create branch {branch}"))?;

        info!(%branch, "branch created");
        Ok(())
    }

    /// Run `git worktree add <path> <branch>` in the main repo.
    fn create_worktree(&self, path: &Path, branch: &str) -> Result<()> {
        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow!("non-UTF8 worktree path"))?;

        let out = StdCommand::new("git")
            .args(["worktree", "add", path_str, branch])
            .current_dir(&self.repo_path)
            .output()
            .context("git worktree add")?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            bail!("git worktree add failed: {stderr}");
        }

        info!(path = ?path, %branch, "worktree created");
        Ok(())
    }

    /// Stage all, commit with a task-tagged message, then push to origin.
    /// Returns the short commit SHA.
    fn commit_and_push(&self, worktree: &Path, task_id: Uuid, branch: &str) -> Result<String> {
        // Stage all changes (new, modified, deleted).
        let out = StdCommand::new("git")
            .args(["add", "-A"])
            .current_dir(worktree)
            .output()
            .context("git add -A")?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            bail!("git add -A failed: {stderr}");
        }

        // Commit. If there is nothing to commit, use HEAD sha instead.
        let commit_message = format!("[T-{task_id}] automated task commit");
        let out = StdCommand::new("git")
            .args(["commit", "-m", &commit_message])
            .current_dir(worktree)
            .output()
            .context("git commit")?;

        let commit_sha = if out.status.success() {
            parse_commit_sha_from_output(&out.stdout)
                .unwrap_or_else(|_| head_sha(worktree).unwrap_or_default())
        } else {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.contains("nothing to commit") || stdout.contains("nothing added") {
                info!(%task_id, "nothing to commit — using HEAD sha");
                head_sha(worktree)?
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                bail!("git commit failed: {stdout} {stderr}");
            }
        };

        info!(%task_id, %commit_sha, "committed");
        self.push(branch)?;

        Ok(commit_sha)
    }

    /// Push `branch` to origin, injecting the access token into the remote URL.
    fn push(&self, branch: &str) -> Result<()> {
        let Some(token) = &self.github_access_token else {
            info!(%branch, "no github_access_token configured, skipping push");
            return Ok(());
        };

        let remote_url = self.authenticated_remote_url(token)?;
        let refspec = format!("refs/heads/{branch}:refs/heads/{branch}");

        let out = StdCommand::new("git")
            .args(["push", &remote_url, &refspec])
            .current_dir(&self.repo_path)
            .output()
            .context("git push")?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            bail!("git push failed: {stderr}");
        }

        info!(%branch, "push complete");
        Ok(())
    }

    /// Run `git worktree remove --force <path>`.
    fn remove_worktree(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }

        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow!("non-UTF8 worktree path"))?;

        let out = StdCommand::new("git")
            .args(["worktree", "remove", "--force", path_str])
            .current_dir(&self.repo_path)
            .output()
            .context("git worktree remove")?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            bail!("git worktree remove failed: {stderr}");
        }

        info!(path = ?path, "worktree removed");
        Ok(())
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn worktree_path(&self, story_id: Uuid) -> PathBuf {
        self.repo_path
            .join(".worktrees")
            .join(format!("story-{story_id}"))
    }

    /// Build an authenticated HTTPS remote URL by injecting the token.
    fn authenticated_remote_url(&self, token: &str) -> Result<String> {
        let out = StdCommand::new("git")
            .args(["remote", "get-url", "origin"])
            .current_dir(&self.repo_path)
            .output()
            .context("git remote get-url origin")?;

        if !out.status.success() {
            bail!("no 'origin' remote configured");
        }

        let url = String::from_utf8(out.stdout)
            .context("remote URL is not UTF-8")?
            .trim()
            .to_string();

        inject_token_into_url(&url, token)
    }

    async fn send(&self, msg: DispatchMessage) {
        if self.dispatch_tx.send(msg).await.is_err() {
            error!("dispatcher channel closed");
        }
    }
}

// ── Free helpers ──────────────────────────────────────────────────────────────

/// Canonical branch name for a story: `story/STORY-{uuid}`.
pub fn story_branch_name(story_id: Uuid) -> String {
    format!("story/STORY-{story_id}")
}

/// Parse the short commit SHA from `git commit` stdout.
/// Example line: `[main abc1234] commit message`
fn parse_commit_sha_from_output(stdout: &[u8]) -> Result<String> {
    let s = String::from_utf8_lossy(stdout);
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix('[') {
            // rest looks like: "main abc1234] commit message"
            if let Some(bracket_end) = rest.find(']') {
                let inside = &rest[..bracket_end]; // "main abc1234"
                if let Some(sha) = inside.split_whitespace().nth(1) {
                    return Ok(sha.to_string());
                }
            }
        }
    }
    Err(anyhow!(
        "could not parse SHA from git commit output: {s:.200}"
    ))
}

/// Return the current HEAD short SHA in `dir`.
fn head_sha(dir: &Path) -> Result<String> {
    let out = StdCommand::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(dir)
        .output()
        .context("git rev-parse HEAD")?;

    if !out.status.success() {
        bail!("git rev-parse HEAD failed");
    }

    Ok(String::from_utf8(out.stdout)
        .context("SHA not UTF-8")?
        .trim()
        .to_string())
}

/// Inject a GitHub personal access token into an HTTPS (or SSH) remote URL.
///
/// - `https://github.com/org/repo.git` → `https://<token>@github.com/org/repo.git`
/// - `git@github.com:org/repo.git`     → `https://<token>@github.com/org/repo.git`
pub fn inject_token_into_url(url: &str, token: &str) -> Result<String> {
    if let Some(rest) = url.strip_prefix("https://") {
        // Strip any existing userinfo (token@…)
        let host_path = rest.split_once('@').map(|(_, r)| r).unwrap_or(rest);
        Ok(format!("https://{token}@{host_path}"))
    } else if let Some(rest) = url.strip_prefix("git@") {
        // git@github.com:org/repo.git → https://<token>@github.com/org/repo.git
        let (host, path) = rest
            .split_once(':')
            .ok_or_else(|| anyhow!("cannot parse SSH remote URL: {url}"))?;
        Ok(format!("https://{token}@{host}/{path}"))
    } else {
        bail!("unsupported remote URL format: {url}")
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command as Cmd;
    use tempfile::TempDir;
    use tokio::sync::mpsc;

    // ── Test repo helpers ─────────────────────────────────────────────────────

    /// Create an initialised git repo with one commit and return the TempDir
    /// (keeps the directory alive) and its path.
    fn make_repo() -> (TempDir, PathBuf) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().to_path_buf();

        Cmd::new("git")
            .args(["init"])
            .current_dir(&path)
            .output()
            .unwrap();
        Cmd::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(&path)
            .output()
            .unwrap();
        Cmd::new("git")
            .args(["config", "user.name", "Test Agent"])
            .current_dir(&path)
            .output()
            .unwrap();

        std::fs::write(path.join("README.md"), "# test repo").unwrap();
        Cmd::new("git")
            .args(["add", "README.md"])
            .current_dir(&path)
            .output()
            .unwrap();
        Cmd::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(&path)
            .output()
            .unwrap();

        (tmp, path)
    }

    fn make_manager(repo_path: PathBuf) -> (GitManager, mpsc::Receiver<DispatchMessage>) {
        let (dispatch_tx, dispatch_rx) = mpsc::channel(16);
        let (_git_tx, git_rx) = mpsc::channel(16);
        let mgr = GitManager::new_with_path(repo_path, None, dispatch_tx, git_rx);
        (mgr, dispatch_rx)
    }

    // ── URL helpers ───────────────────────────────────────────────────────────

    #[test]
    fn inject_token_https() {
        let url = inject_token_into_url("https://github.com/org/repo.git", "mytoken").unwrap();
        assert_eq!(url, "https://mytoken@github.com/org/repo.git");
    }

    #[test]
    fn inject_token_https_replaces_existing_userinfo() {
        let url =
            inject_token_into_url("https://oldtoken@github.com/org/repo.git", "newtoken").unwrap();
        assert_eq!(url, "https://newtoken@github.com/org/repo.git");
    }

    #[test]
    fn inject_token_ssh() {
        let url = inject_token_into_url("git@github.com:org/repo.git", "tok").unwrap();
        assert_eq!(url, "https://tok@github.com/org/repo.git");
    }

    #[test]
    fn inject_token_unsupported_scheme() {
        assert!(inject_token_into_url("ftp://example.com/repo.git", "tok").is_err());
    }

    // ── Branch operations ─────────────────────────────────────────────────────

    #[test]
    fn create_branch_succeeds() {
        let (_tmp, repo_path) = make_repo();
        let (mgr, _rx) = make_manager(repo_path.clone());

        let story_id = Uuid::now_v7();
        let branch = story_branch_name(story_id);
        mgr.ensure_branch(&branch).unwrap();

        // Verify via git
        let out = Cmd::new("git")
            .args(["branch", "--list", &branch])
            .current_dir(&repo_path)
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains(&branch), "branch not found: {stdout}");
    }

    #[test]
    fn create_branch_idempotent() {
        let (_tmp, repo_path) = make_repo();
        let (mgr, _rx) = make_manager(repo_path.clone());

        let story_id = Uuid::now_v7();
        let branch = story_branch_name(story_id);

        mgr.ensure_branch(&branch).unwrap();
        // Second call must not error
        mgr.ensure_branch(&branch).unwrap();
    }

    // ── Worktree operations ───────────────────────────────────────────────────

    #[test]
    fn ensure_worktree_creates_dir() {
        let (_tmp, repo_path) = make_repo();
        let (mgr, _rx) = make_manager(repo_path.clone());

        let story_id = Uuid::now_v7();
        let worktree = mgr.ensure_worktree(story_id).unwrap();

        assert!(worktree.exists(), "worktree path should exist");
        // Path must follow the deterministic convention
        assert_eq!(
            worktree,
            repo_path
                .join(".worktrees")
                .join(format!("story-{story_id}"))
        );
    }

    #[test]
    fn ensure_worktree_idempotent() {
        let (_tmp, repo_path) = make_repo();
        let (mgr, _rx) = make_manager(repo_path);

        let story_id = Uuid::now_v7();
        let wt1 = mgr.ensure_worktree(story_id).unwrap();
        let wt2 = mgr.ensure_worktree(story_id).unwrap();
        assert_eq!(wt1, wt2);
    }

    #[test]
    fn remove_worktree_succeeds() {
        let (_tmp, repo_path) = make_repo();
        let (mgr, _rx) = make_manager(repo_path.clone());

        let story_id = Uuid::now_v7();
        let wt = mgr.ensure_worktree(story_id).unwrap();
        assert!(wt.exists());

        mgr.remove_worktree(&wt).unwrap();
        assert!(!wt.exists());
    }

    // ── Commit operation ──────────────────────────────────────────────────────

    #[test]
    fn commit_and_push_no_token_returns_sha() {
        let (_tmp, repo_path) = make_repo();
        let (mgr, _rx) = make_manager(repo_path.clone());

        let story_id = Uuid::now_v7();
        let task_id = Uuid::now_v7();
        let wt = mgr.ensure_worktree(story_id).unwrap();

        // Write a file in the worktree
        std::fs::write(wt.join("output.txt"), "hello from claude").unwrap();

        let branch = story_branch_name(story_id);
        let sha = mgr.commit_and_push(&wt, task_id, &branch).unwrap();

        assert!(!sha.is_empty(), "SHA must not be empty");

        // Commit message should reference the task
        let log = Cmd::new("git")
            .args(["log", "--oneline", "-1"])
            .current_dir(&wt)
            .output()
            .unwrap();
        let log_str = String::from_utf8_lossy(&log.stdout);
        assert!(
            log_str.contains(&task_id.to_string()),
            "commit message missing task ID"
        );
    }

    #[test]
    fn commit_nothing_to_commit_returns_head_sha() {
        let (_tmp, repo_path) = make_repo();
        let (mgr, _rx) = make_manager(repo_path.clone());

        let story_id = Uuid::now_v7();
        let task_id = Uuid::now_v7();
        let wt = mgr.ensure_worktree(story_id).unwrap();

        let branch = story_branch_name(story_id);
        // No changes — should still return HEAD sha without error
        let sha = mgr.commit_and_push(&wt, task_id, &branch).unwrap();
        assert!(!sha.is_empty());
    }

    // ── Dispatcher integration ────────────────────────────────────────────────

    #[tokio::test]
    async fn handle_ensure_worktree_sends_worktree_ready() {
        let (_tmp, repo_path) = make_repo();
        let (dispatch_tx, mut dispatch_rx) = mpsc::channel(16);
        let (git_tx, git_rx) = mpsc::channel(16);
        let mgr = GitManager::new_with_path(repo_path, None, dispatch_tx, git_rx);

        let story_id = Uuid::now_v7();
        let task_id = Uuid::now_v7();
        let ctx = shared::types::TaskContext {
            task_description: "do the thing".into(),
            story_decisions: vec![],
            sibling_decisions: vec![],
            knowledge: vec![],
        };

        git_tx
            .send(GitMessage::EnsureWorktree {
                story_id,
                task_id,
                session_id: "sess-1".into(),
                context: ctx.clone(),
            })
            .await
            .unwrap();

        // Process one message
        drop(git_tx); // close so run() exits
        mgr.run().await;

        let msg = dispatch_rx.try_recv().expect("expected WorktreeReady");
        assert!(
            matches!(msg, DispatchMessage::WorktreeReady { task_id: tid, .. } if tid == task_id)
        );
    }
}
