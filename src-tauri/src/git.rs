use serde::Serialize;
use std::path::Path;
use std::process::{Command, Stdio};

pub const GIT_OUTPUT_LINE_LIMIT: usize = 500;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    pub hash: String,
    pub short_hash: String,
    pub subject: String,
    pub author: String,
    pub date: String,
    pub parents: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangedFile {
    pub index: String,
    pub worktree: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoStatus {
    pub branch: String,
    pub ahead_behind: String,
    pub files: Vec<ChangedFile>,
    pub staged: usize,
    pub unstaged: usize,
    pub untracked: usize,
    pub clean: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitResult {
    pub hash: String,
    pub short_hash: String,
}


#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDiff {
    pub path: String,
    pub status: String,
    pub old_content: String,
    pub new_content: String,
}

fn git_cmd() -> Command {
    #[allow(unused_mut)]
    let mut cmd = Command::new("git");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

fn git_available() -> bool {
    git_cmd()
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

pub fn is_git_repo(cwd: &str) -> bool {
    git(cwd, &["rev-parse", "--git-dir"]).is_ok()
}

pub fn has_commits(cwd: &str) -> bool {
    git(cwd, &["rev-parse", "HEAD"]).is_ok()
}

pub fn repo_root(cwd: &str) -> Result<String, String> {
    if !git_available() {
        return Err("Git is not installed".into());
    }
    if git(cwd, &["rev-parse", "--git-dir"]).is_err() {
        return Err("Not a git repository".into());
    }
    let out = git(cwd, &["rev-parse", "--show-toplevel"])?;
    Ok(out.trim().to_string())
}

pub fn git_init(path: &str) -> Result<String, String> {
    if !git_available() {
        return Err("Git is not installed".into());
    }
    if std::path::Path::new(path).is_file() {
        return Err(format!("{path} is a file, not a directory"));
    }
    let _ = std::fs::create_dir_all(path);
    git(path, &["init"]).map(|s| s.trim().to_string())
}

fn git(cwd: &str, args: &[&str]) -> Result<String, String> {
    let out = git_cmd()
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("Error running git: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    if out.status.success() {
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        let trimmed = if stdout.trim().is_empty() { stderr.trim() } else { stdout.trim() };
        Err(if trimmed.is_empty() {
            format!("git {} failed", args.join(" "))
        } else {
            trimmed.to_string()
        })
    }
}

fn truncate_output(content: &str) -> String {
    if content.lines().count() <= GIT_OUTPUT_LINE_LIMIT {
        return content.to_string();
    }
    let lines: Vec<&str> = content.lines().collect();
    let before = (GIT_OUTPUT_LINE_LIMIT as f32 * 0.2) as usize;
    let after = GIT_OUTPUT_LINE_LIMIT - before;
    let omitted = format!("[...{} lines omitted...]", lines.len() - GIT_OUTPUT_LINE_LIMIT);
    let mut out: Vec<&str> = lines[..before].to_vec();
    out.push(&omitted);
    out.extend_from_slice(&lines[lines.len() - after..]);
    out.join("\n")
}

/// Mirrors getGitDiff in cline: staged diff first, fallback to unstaged + untracked.
pub fn git_diff(cwd: &str, staged_only: bool) -> Result<String, String> {
    if !git_available() {
        return Err("Git is not installed".into());
    }
    if !is_git_repo(cwd) {
        return Err("Not a git repository".into());
    }

    let staged = git(cwd, &["--no-pager", "diff", "--staged", "--diff-filter=d"])?;
    let mut diff = staged.trim().to_string();

    if !staged_only && diff.is_empty() && has_commits(cwd) {
        let unstaged = git(cwd, &["--no-pager", "diff", "HEAD", "--diff-filter=d"])?;
        diff = unstaged.trim().to_string();
    }

    if !staged_only {
        let untracked = untracked_diff(cwd)?;
        if !untracked.is_empty() {
            diff = if diff.is_empty() { untracked } else { format!("{diff}\n\n{untracked}") };
        }
    }

    if diff.is_empty() {
        return Err("No changes in workspace for commit message".into());
    }

    Ok(truncate_output(&diff))
}

/// `git diff --no-index` for each untracked file against /dev/null (works on
/// Windows git too — the `/dev/null` path is handled by git's diff machinery).
fn untracked_diff(cwd: &str) -> Result<String, String> {
    let list = git(cwd, &["ls-files", "--others", "--exclude-standard", "-z"])?;
    let files: Vec<&str> = list.split('\0').filter(|f| !f.is_empty()).collect();
    if files.is_empty() {
        return Ok(String::new());
    }

    let mut diffs: Vec<String> = Vec::new();
    for file in files {
        let out = git_cmd()
            .args(["--no-pager", "diff", "--no-index", "--diff-filter=d", "--"])
            .arg("/dev/null")
            .arg(file)
            .current_dir(cwd)
            .output()
            .map_err(|e| format!("Error diffing file: {e}"))?;
        // exit 1 = files differ (normal); anything else = real git error
        if !out.status.success() && out.status.code() != Some(1) {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(format!("git diff --no-index failed: {}", stderr.trim()));
        }
        let trimmed = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !trimmed.is_empty() {
            diffs.push(trimmed);
        }
    }
    Ok(diffs.join("\n\n"))
}

pub fn git_status(cwd: &str) -> Result<RepoStatus, String> {
    let out = git(cwd, &["status", "--short", "--branch"])?;
    let mut branch = String::new();
    let mut ahead_behind = String::new();
    let mut files: Vec<ChangedFile> = Vec::new();
    let mut staged = 0;
    let mut unstaged = 0;
    let mut untracked = 0;

    for raw in out.lines() {
        if raw.starts_with("##") {
            let rest = raw[2..].trim().to_string();
            let (b, ab) = match rest.split_once("...") {
                Some((b, ab)) => (b.to_string(), ab.replace(['[', ']'], "")),
                None => (rest.clone(), String::new()),
            };
            branch = b;
            ahead_behind = ab;
            continue;
        }
        let mut chars = raw.chars();
        let x = chars.next().unwrap_or(' ');
        let y = chars.next().unwrap_or(' ');
        let path = raw.chars().skip(3).collect::<String>().trim().to_string();
        files.push(ChangedFile { index: x.to_string(), worktree: y.to_string(), path });
        if x != ' ' && x != '?' {
            staged += 1;
        }
        if y != ' ' {
            unstaged += 1;
        }
        if x == '?' || y == '?' {
            untracked += 1;
        }
    }

    Ok(RepoStatus {
        branch,
        ahead_behind,
        clean: files.is_empty(),
        files,
        staged,
        unstaged,
        untracked,
    })
}

pub fn git_log(cwd: &str, limit: usize) -> Result<Vec<Commit>, String> {
    let n = limit.max(1);
    let fmt = "%H%x1f%h%x1f%P%x1f%an%x1f%ad%x1f%s%x1e";
    let out = git(
        cwd,
        &["log", "--all", "--topo-order", "--date=iso-strict", &format!("-n{n}"), &format!("--pretty=format:{fmt}")],
    )?;
    Ok(out
        .split('\x1e')
        .filter(|r| !r.trim().is_empty())
        .filter_map(|rec| {
            let fields: Vec<&str> = rec.split('\x1f').collect();
            if fields.len() < 6 {
                return None;
            }
            let parents = fields[2].split(' ').filter(|p| !p.is_empty()).map(|s| s.to_string()).collect();
            Some(Commit {
                hash: fields[0].trim().to_string(),
                short_hash: fields[1].trim().to_string(),
                parents,
                author: fields[3].trim().to_string(),
                date: fields[4].trim().to_string(),
                subject: fields[5].trim().to_string(),
            })
        })
        .collect())
}

fn head_hash(cwd: &str) -> String {
    git(cwd, &["rev-parse", "HEAD"]).map(|s| s.trim().to_string()).unwrap_or_default()
}

fn spectral(deg: usize, max_degree: usize) -> (f32, String) {
    let ratio = deg as f32 / max_degree.max(1) as f32;
    let size = 2.0 + ratio * 10.0;
    let color = match ratio {
        r if r < 0.06 => "#ff6050".to_string(),
        r if r < 0.16 => "#ffa060".to_string(),
        r if r < 0.34 => "#ffe080".to_string(),
        r if r < 0.56 => "#fff0c0".to_string(),
        r if r < 0.78 => "#e8e8ff".to_string(),
        r if r < 0.95 => "#c0d0ff".to_string(),
        _ => "#80a0ff".to_string(),
    };
    (size, color)
}

/// Old/new content of each file changed by a commit (A/M/D), VSCode-git style.
pub fn git_show_commit(cwd: &str, hash: &str) -> Result<Vec<FileDiff>, String> {
    let rev = hash.trim();
    if rev.is_empty() {
        return Err("Empty hash".into());
    }
    let out = git(cwd, &["show", "--no-renames", "--format=", "--name-status", "-r", rev])?;
    let mut diffs = Vec::new();
    for line in out.lines() {
        let mut parts = line.splitn(2, '\t');
        let status = parts.next().unwrap_or("").trim().to_string();
        let path = parts.next().map(|s| s.trim().to_string()).unwrap_or_default();
        if path.is_empty() {
            continue;
        }
        let old_content = if status == "A" { String::new() } else { blob(cwd, &format!("{rev}^:{path}")) };
        let new_content = if status == "D" { String::new() } else { blob(cwd, &format!("{rev}:{path}")) };
        diffs.push(FileDiff { path, status, old_content, new_content });
    }
    Ok(diffs)
}

fn blob(cwd: &str, spec: &str) -> String {
    let out = git_cmd()
        .args(["-c", "core.quotepath=false", "show", spec])
        .current_dir(cwd)
        .output();
    match out {
        Ok(o) if o.status.success() => {
            if o.stdout.contains(&0) {
                "[binary file]".to_string()
            } else {
                String::from_utf8_lossy(&o.stdout).to_string()
            }
        }
        _ => String::new(),
    }
}

pub fn git_commit(cwd: &str, message: &str) -> Result<CommitResult, String> {
    let msg = message.trim();
    if msg.is_empty() {
        return Err("Commit message is empty".into());
    }
    git(cwd, &["add", "-A"])?;
    let out = git_cmd()
        .args(["commit", "-m"])
        .arg(msg)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("Error committing: {e}"))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        return Err(if stderr.trim().is_empty() {
            "git commit failed".to_string()
        } else {
            stderr.trim().to_string()
        });
    }
    let short = git(cwd, &["rev-parse", "--short", "HEAD"]).map(|s| s.trim().to_string()).unwrap_or_default();
    let full = git(cwd, &["rev-parse", "HEAD"]).map(|s| s.trim().to_string()).unwrap_or_default();
    Ok(CommitResult { hash: full, short_hash: short })
}

pub fn repo_name(cwd: &str) -> String {
    Path::new(cwd)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| cwd.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    fn git(cwd: &Path, args: &[&str]) {
        let ok = Command::new("git").args(args).current_dir(cwd).status().unwrap().success();
        assert!(ok, "git {:?} failed", args);
    }

    #[test]
    fn diff_status_log_graph_commit_workflow() {
        let dir = std::env::temp_dir().join(format!("commitgen-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);
        git(&dir, &["config", "user.email", "t@t.t"]);
        git(&dir, &["config", "user.name", "T"]);
        fs::write(dir.join("a.txt"), "one\n").unwrap();
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-qm", "first"]);
        fs::write(dir.join("a.txt"), "two\n").unwrap();
        fs::write(dir.join("b.txt"), "new\n").unwrap();

        let s = git_status(dir.to_str().unwrap()).unwrap();
        assert!(s.unstaged >= 1 && s.untracked >= 1, "status counts wrong: {s:?}");

        let diff = git_diff(dir.to_str().unwrap(), false).unwrap();
        assert!(diff.contains("a.txt") && diff.contains("b.txt"), "untracked diff missing");

        let log = git_log(dir.to_str().unwrap(), 10).unwrap();
        assert_eq!(log.len(), 1, "expected 1 commit");
        assert_eq!(log[0].subject, "first");
        assert!(log[0].parents.is_empty());


        let c = git_commit(dir.to_str().unwrap(), "second: change").unwrap();
        assert!(!c.short_hash.is_empty());
        assert!(git_status(dir.to_str().unwrap()).unwrap().clean, "should be clean after commit");
        assert_eq!(git_log(dir.to_str().unwrap(), 10).unwrap().len(), 2);

        let show = git_show_commit(dir.to_str().unwrap(), &c.hash).unwrap();
        assert_eq!(show.len(), 2, "second commit should list 2 files: {show:?}");
        let a = show.iter().find(|d| d.path == "a.txt").unwrap();
        assert_eq!(a.old_content.trim(), "one", "old side wrong");
        assert_eq!(a.new_content.trim(), "two", "new side wrong");
        let b = show.iter().find(|d| d.path == "b.txt").unwrap();
        assert_eq!(b.status, "A", "b.txt should be Added");
        assert!(b.old_content.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }
}