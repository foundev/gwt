use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn gwt_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_gwt"))
}

/// Creates a temp dir with a git repo containing one commit.
fn setup_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let path = dir.path();

    Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(path)
        .output()
        .unwrap();

    std::fs::write(path.join("file.txt"), "hello").unwrap();

    Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(path)
        .output()
        .unwrap();

    dir
}

fn run_gwt(repo: &TempDir, args: &[&str]) -> std::process::Output {
    Command::new(gwt_bin())
        .args(args)
        .current_dir(repo.path())
        .output()
        .unwrap()
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

// ── add ──

#[test]
fn add_with_name_creates_worktree() {
    let repo = setup_repo();
    let output = run_gwt(&repo, &["add", "my-feature"]);

    assert!(output.status.success());

    let wt_path = repo.path().join(".worktrees/my-feature");
    assert!(wt_path.exists(), "worktree directory should exist");
    assert!(wt_path.join(".git").exists(), "should be a git worktree");
    assert_eq!(stdout(&output), wt_path.display().to_string());
}

#[test]
fn add_without_name_generates_random_name() {
    let repo = setup_repo();
    let output = run_gwt(&repo, &["add"]);

    assert!(output.status.success());

    let path = stdout(&output);
    assert!(
        PathBuf::from(&path).exists(),
        "generated worktree should exist"
    );

    // Name should be adjective-verb-noun (3 parts)
    let name = PathBuf::from(&path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let parts: Vec<&str> = name.split('-').collect();
    assert!(
        parts.len() >= 3,
        "random name '{name}' should have at least 3 parts"
    );
}

#[test]
fn add_creates_gitignore_entry() {
    let repo = setup_repo();
    run_gwt(&repo, &["add", "test-ignore"]);

    let gitignore = std::fs::read_to_string(repo.path().join(".gitignore")).unwrap();
    assert!(
        gitignore.contains(".worktrees"),
        ".gitignore should contain .worktrees"
    );
}

#[test]
fn add_does_not_duplicate_gitignore_entry() {
    let repo = setup_repo();
    run_gwt(&repo, &["add", "first"]);
    run_gwt(&repo, &["add", "second"]);

    let gitignore = std::fs::read_to_string(repo.path().join(".gitignore")).unwrap();
    let count = gitignore
        .lines()
        .filter(|l| l.trim() == ".worktrees")
        .count();
    assert_eq!(count, 1, ".worktrees should appear only once in .gitignore");
}

#[test]
fn add_existing_branch_reuses_it() {
    let repo = setup_repo();

    // Create a branch manually
    Command::new("git")
        .args(["branch", "existing-branch"])
        .current_dir(repo.path())
        .output()
        .unwrap();

    let output = run_gwt(&repo, &["add", "existing-branch"]);
    assert!(output.status.success());
    assert!(repo.path().join(".worktrees/existing-branch").exists());
}

#[test]
fn add_duplicate_name_fails() {
    let repo = setup_repo();
    run_gwt(&repo, &["add", "dup"]);
    let output = run_gwt(&repo, &["add", "dup"]);

    assert!(!output.status.success());
}

// ── ls ──

#[test]
fn ls_shows_main_worktree() {
    let repo = setup_repo();
    let output = run_gwt(&repo, &["ls"]);

    assert!(output.status.success());
    let out = stdout(&output);
    assert!(
        out.contains("master") || out.contains("main"),
        "should show main branch"
    );
}

#[test]
fn ls_shows_added_worktrees() {
    let repo = setup_repo();
    run_gwt(&repo, &["add", "feat-a"]);
    run_gwt(&repo, &["add", "feat-b"]);

    let output = run_gwt(&repo, &["ls"]);
    let out = stdout(&output);

    assert!(out.contains("feat-a"), "should list feat-a");
    assert!(out.contains("feat-b"), "should list feat-b");
}

#[test]
fn ls_marks_current_worktree() {
    let repo = setup_repo();
    let output = run_gwt(&repo, &["ls"]);
    let out = stdout(&output);

    assert!(out.contains("*"), "should mark current worktree with *");
}

// ── cd ──

#[test]
fn cd_outputs_worktree_path() {
    let repo = setup_repo();
    run_gwt(&repo, &["add", "target-wt"]);

    let output = run_gwt(&repo, &["cd", "target-wt"]);
    assert!(output.status.success());

    let expected = repo.path().join(".worktrees/target-wt");
    assert_eq!(stdout(&output), expected.display().to_string());
}

#[test]
fn cd_nonexistent_fails() {
    let repo = setup_repo();
    let output = run_gwt(&repo, &["cd", "nope"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("introuvable"));
}

// ── rm ──

#[test]
fn rm_removes_worktree() {
    let repo = setup_repo();
    run_gwt(&repo, &["add", "to-remove"]);

    let wt_path = repo.path().join(".worktrees/to-remove");
    assert!(wt_path.exists());

    let output = run_gwt(&repo, &["rm", "to-remove"]);
    assert!(output.status.success());
    assert!(!wt_path.exists(), "worktree directory should be gone");
}

#[test]
fn rm_nonexistent_fails() {
    let repo = setup_repo();
    let output = run_gwt(&repo, &["rm", "ghost"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("introuvable"));
}

#[test]
fn rm_no_arg_fails() {
    let repo = setup_repo();
    let output = run_gwt(&repo, &["rm"]);

    assert!(!output.status.success());
}

// ── rm then ls ──

#[test]
fn rm_then_ls_no_longer_shows_removed() {
    let repo = setup_repo();
    run_gwt(&repo, &["add", "ephemeral"]);
    run_gwt(&repo, &["rm", "ephemeral"]);

    let output = run_gwt(&repo, &["ls"]);
    let out = stdout(&output);
    assert!(
        !out.contains("ephemeral"),
        "removed worktree should not appear in ls"
    );
}

// ── shell-init ──

#[test]
fn shell_init_outputs_function() {
    let repo = setup_repo();
    let output = run_gwt(&repo, &["shell-init"]);

    assert!(output.status.success());
    let out = stdout(&output);
    assert!(out.contains("gwt()"), "should define a gwt shell function");
    assert!(out.contains("builtin cd"), "should use builtin cd");
}

// ── help ──

#[test]
fn help_shows_usage() {
    let repo = setup_repo();

    for flag in &["help", "--help", "-h"] {
        let output = run_gwt(&repo, &[flag]);
        let err = stderr(&output);
        assert!(err.contains("add"), "help should mention add command");
        assert!(err.contains("rm"), "help should mention rm command");
        assert!(err.contains("ls"), "help should mention ls command");
        assert!(err.contains("cd"), "help should mention cd command");
    }
}

#[test]
fn no_args_shows_usage() {
    let repo = setup_repo();
    let output = run_gwt(&repo, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("gwt"));
}

#[test]
fn unknown_command_fails() {
    let repo = setup_repo();
    let output = run_gwt(&repo, &["frobnicate"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("inconnue"));
}
