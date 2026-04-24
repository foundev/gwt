# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build                    # build
cargo test                     # run all tests (integration tests in tests/cli.rs)
cargo test <test_name>         # run a single test
cargo clippy -- -D warnings    # lint (CI treats warnings as errors)
cargo fmt -- --check           # check formatting
cargo fmt                      # auto-format
```

CI runs: check, test, clippy, fmt. All must pass.

## Architecture

Single-binary Rust CLI (`src/main.rs`) that wraps `git worktree` commands. Zero runtime dependencies — only uses `std`.

**Stdout/stderr contract:** For `add` and `cd`, the worktree path goes to **stdout** (captured by the shell wrapper for `cd`). All other output (git messages, user-facing info, errors) goes to **stderr**. This separation is critical — breaking it will break the shell integration.

**Worktree location:** Worktrees are created under `<repo>/.worktrees/<name>/`. The `.worktrees` entry is auto-added to `.gitignore` on first `add`.

**Shell integration:** A subprocess cannot change the parent shell's directory. `gwt shell-init` outputs a shell function that wraps the binary — it captures stdout from `add`/`cd` and runs `builtin cd` on it. Users source this via `eval "$(gwt shell-init)"`.

**Worktree resolution:** `find_worktree()` matches by branch name first, then by directory name. `main_worktree_root()` uses `git rev-parse --git-common-dir` to always resolve to the main repo root, even when run from a secondary worktree.

**Random naming:** When `add` is called without a name, generates `adjective-verb-noun` names (word lists are inline constants).

## Testing

Tests are integration tests (`tests/cli.rs`) that spin up real temp git repos via `tempfile::TempDir`. Each test gets an isolated repo with one initial commit. Tests run the compiled binary via `std::process::Command`.
