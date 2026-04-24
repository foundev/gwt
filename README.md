# gwt

Simple git worktree manager. Create, switch, list, and remove worktrees without remembering the full `git worktree` syntax.

## Install

```bash
cargo install --git https://github.com/foundev/gwt
```

Or download a binary from the [releases page](https://github.com/foundev/gwt/releases).

## Setup

Add this to your `.bashrc` or `.zshrc` to enable automatic directory switching:

```bash
eval "$(gwt shell-init)"
```

Without this, `gwt add` and `gwt cd` will print the path but won't change your shell directory.

## Usage

```bash
gwt add my-feature   # create a worktree and cd into it
gwt add              # same, with a random name (e.g. bold-dancing-ferris)
gwt ls               # list all worktrees
gwt cd my-feature    # switch to a worktree
gwt rm my-feature    # remove a worktree
```

Worktrees are stored in `.worktrees/` inside the repo (auto-added to `.gitignore`).

```
$ gwt ls
* master      ~/code/myproject
  my-feature  ~/code/myproject/.worktrees/my-feature
  bugfix-42   ~/code/myproject/.worktrees/bugfix-42
```

## License

[GPL-3.0](LICENSE)
