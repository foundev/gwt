use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process::{self, Command, Stdio};
use std::time::SystemTime;

const ADJECTIVES: &[&str] = &[
    "bold", "brave", "bright", "calm", "clever", "cool", "cosmic", "cozy",
    "crisp", "dapper", "eager", "fancy", "fierce", "fluffy", "funky",
    "gentle", "giddy", "grand", "groovy", "happy", "humble", "jazzy",
    "jolly", "keen", "lively", "lucky", "mellow", "mighty", "neat", "nimble",
    "peppy", "plucky", "proud", "quick", "quiet", "rapid", "rustic",
    "sharp", "shiny", "slick", "snappy", "snazzy", "spicy", "steady",
    "sunny", "swift", "witty", "zany", "zesty", "zippy",
];

const VERBS: &[&str] = &[
    "blazing", "bouncing", "buzzing", "chasing", "climbing", "crafting",
    "dancing", "dashing", "dazzling", "diving", "dreaming", "drifting",
    "flipping", "floating", "flying", "frolicking", "gliding", "grinning",
    "hiking", "hopping", "humming", "jumping", "juggling", "leaping",
    "marching", "prancing", "racing", "roaming", "rolling", "running",
    "sailing", "singing", "skating", "skipping", "soaring", "spinning",
    "sprinting", "strolling", "surfing", "swooping", "tapping", "trekking",
    "tumbling", "twirling", "vaulting", "wandering", "weaving", "whirling",
    "whistling", "zooming",
];

const NOUNS: &[&str] = &[
    "babbage", "banjo", "blossom", "breeze", "cobalt", "comet", "compass",
    "curie", "dijkstra", "donkey", "euler", "falcon", "ferris", "feynman",
    "gauss", "grove", "hopper", "lantern", "lamport", "maple", "moose",
    "newton", "otter", "panda", "phoenix", "pluto", "quasar", "raven",
    "rocket", "sequoia", "shuttle", "spark", "sphinx", "summit", "tesla",
    "thistle", "tiger", "trinket", "turing", "walrus", "widget", "willow",
    "zenith",
];

fn random_name() -> String {
    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    // Simple hash to spread entropy across picks
    let h1 = (seed ^ (seed >> 17)).wrapping_mul(0x9E3779B97F4A7C15) as usize;
    let h2 = (seed ^ (seed >> 13)).wrapping_mul(0xBF58476D1CE4E5B9) as usize;
    let h3 = (seed ^ (seed >> 11)).wrapping_mul(0x94D049BB133111EB) as usize;

    let adj = ADJECTIVES[h1 % ADJECTIVES.len()];
    let verb = VERBS[h2 % VERBS.len()];
    let noun = NOUNS[h3 % NOUNS.len()];
    format!("{adj}-{verb}-{noun}")
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "add" => cmd_add(args.get(2).map(String::as_str)),
        "rm" | "remove" => match args.get(2) {
            Some(name) => cmd_rm(name),
            None => {
                eprintln!("Usage: gwt rm <nom>");
                process::exit(1);
            }
        },
        "ls" | "list" => cmd_ls(),
        "cd" => match args.get(2) {
            Some(name) => cmd_cd(name),
            None => {
                eprintln!("Usage: gwt cd <nom>");
                process::exit(1);
            }
        },
        "shell-init" => cmd_shell_init(),
        "-h" | "--help" | "help" => usage(),
        other => {
            eprintln!("Commande inconnue : {other}");
            usage();
            process::exit(1);
        }
    }
}

fn usage() {
    eprintln!(
        "gwt - gestionnaire de git worktrees

Commandes :
  add [nom]       Creer un nouveau worktree (nom auto-genere si omis)
  rm <nom>        Supprimer un worktree
  ls              Lister les worktrees
  cd <nom>        Aller dans un worktree
  shell-init      Afficher la fonction shell a sourcer

Pour activer le cd automatique, ajoutez a votre .bashrc/.zshrc :
  eval \"$(gwt shell-init)\""
    );
}

/// Returns the root of the main worktree, even when called from a secondary worktree.
fn main_worktree_root() -> PathBuf {
    let output = Command::new("git")
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .output()
        .unwrap_or_else(|e| {
            eprintln!("Impossible d'executer git : {e}");
            process::exit(1);
        });

    if !output.status.success() {
        eprintln!("Pas dans un depot git");
        process::exit(1);
    }

    let git_common = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
    git_common
        .parent()
        .unwrap_or_else(|| {
            eprintln!("Impossible de determiner la racine du depot");
            process::exit(1);
        })
        .to_path_buf()
}

struct Worktree {
    path: PathBuf,
    branch: Option<String>,
}

fn list_worktrees() -> Vec<Worktree> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .output()
        .unwrap_or_else(|e| {
            eprintln!("Impossible d'executer git : {e}");
            process::exit(1);
        });

    let text = String::from_utf8_lossy(&output.stdout);
    let mut worktrees = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    let mut current_branch: Option<String> = None;

    for line in text.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            if let Some(p) = current_path.take() {
                worktrees.push(Worktree {
                    path: p,
                    branch: current_branch.take(),
                });
            }
            current_path = Some(PathBuf::from(path));
        } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
            current_branch = Some(branch.to_string());
        } else if line.is_empty() {
            if let Some(p) = current_path.take() {
                worktrees.push(Worktree {
                    path: p,
                    branch: current_branch.take(),
                });
            }
        }
    }
    if let Some(p) = current_path.take() {
        worktrees.push(Worktree {
            path: p,
            branch: current_branch.take(),
        });
    }

    worktrees
}

fn find_worktree(name: &str) -> Option<Worktree> {
    let worktrees = list_worktrees();

    // Match by branch name first
    for wt in &worktrees {
        if wt.branch.as_deref() == Some(name) {
            return Some(Worktree {
                path: wt.path.clone(),
                branch: wt.branch.clone(),
            });
        }
    }

    // Then by directory name
    for wt in &worktrees {
        if let Some(dir_name) = wt.path.file_name().and_then(|n| n.to_str()) {
            if dir_name == name {
                return Some(Worktree {
                    path: wt.path.clone(),
                    branch: wt.branch.clone(),
                });
            }
        }
    }

    None
}

fn branch_exists(name: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", &format!("refs/heads/{name}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn cmd_add(name: Option<&str>) {
    let root = main_worktree_root();
    let wt_dir = root.join(".worktrees");

    let name = match name {
        Some(n) => n.to_string(),
        None => {
            let existing: Vec<String> = list_worktrees()
                .iter()
                .filter_map(|wt| wt.branch.clone())
                .collect();
            let mut name = random_name();
            while existing.contains(&name) {
                name = random_name();
            }
            name
        }
    };

    if !wt_dir.exists() {
        std::fs::create_dir_all(&wt_dir).unwrap_or_else(|e| {
            eprintln!("Impossible de creer {}: {e}", wt_dir.display());
            process::exit(1);
        });
    }

    // Add .worktrees to .gitignore if not already there
    let gitignore = root.join(".gitignore");
    let should_add = if gitignore.exists() {
        let content = std::fs::read_to_string(&gitignore).unwrap_or_default();
        !content.lines().any(|l| l.trim() == ".worktrees")
    } else {
        true
    };
    if should_add {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&gitignore)
            .unwrap_or_else(|e| {
                eprintln!("Impossible d'ecrire .gitignore: {e}");
                process::exit(1);
            });
        writeln!(f, ".worktrees").ok();
        eprintln!("Ajout de .worktrees a .gitignore");
    }

    let wt_path = wt_dir.join(&name);

    let args: Vec<String> = if branch_exists(&name) {
        vec![
            "worktree".into(),
            "add".into(),
            wt_path.display().to_string(),
            name.clone(),
        ]
    } else {
        vec![
            "worktree".into(),
            "add".into(),
            wt_path.display().to_string(),
            "-b".into(),
            name.clone(),
        ]
    };

    let output = Command::new("git")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .unwrap_or_else(|e| {
            eprintln!("Impossible d'executer git : {e}");
            process::exit(1);
        });

    // Forward git's stdout to stderr so user sees it but shell function doesn't capture it
    if !output.stdout.is_empty() {
        std::io::stderr().write_all(&output.stdout).ok();
    }

    if !output.status.success() {
        process::exit(1);
    }

    // Only the path goes to stdout — the shell wrapper captures this for cd
    println!("{}", wt_path.display());
}

fn cmd_rm(name: &str) {
    let wt = find_worktree(name).unwrap_or_else(|| {
        eprintln!("Worktree '{name}' introuvable");
        process::exit(1);
    });

    let status = Command::new("git")
        .args(["worktree", "remove", wt.path.to_str().unwrap()])
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Impossible d'executer git : {e}");
            process::exit(1);
        });

    if !status.success() {
        process::exit(1);
    }

    eprintln!("Worktree '{name}' supprime");
}

fn shorten_path(path: &std::path::Path) -> String {
    if let Some(home) = env::var_os("HOME") {
        if let Ok(rest) = path.strip_prefix(&home) {
            return format!("~/{}", rest.display());
        }
    }
    path.display().to_string()
}

fn current_worktree() -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if output.status.success() {
        Some(PathBuf::from(
            String::from_utf8_lossy(&output.stdout).trim(),
        ))
    } else {
        None
    }
}

fn cmd_ls() {
    let worktrees = list_worktrees();

    if worktrees.is_empty() {
        println!("Aucun worktree");
        return;
    }

    let current = current_worktree();

    let labels: Vec<String> = worktrees
        .iter()
        .map(|wt| wt.branch.clone().unwrap_or_else(|| "(detache)".into()))
        .collect();

    let max_len = labels.iter().map(|l| l.len()).max().unwrap_or(0);

    for (wt, label) in worktrees.iter().zip(&labels) {
        let marker = if current.as_deref() == Some(&*wt.path) {
            "*"
        } else {
            " "
        };
        println!(
            "{} {:<width$}  {}",
            marker,
            label,
            shorten_path(&wt.path),
            width = max_len
        );
    }
}

fn cmd_cd(name: &str) {
    let wt = find_worktree(name).unwrap_or_else(|| {
        eprintln!("Worktree '{name}' introuvable");
        process::exit(1);
    });

    println!("{}", wt.path.display());
}

fn cmd_shell_init() {
    print!(
        r#"# gwt - integration shell
# Ajoutez ceci a votre .bashrc ou .zshrc :
#   eval "$(gwt shell-init)"

gwt() {{
    if [ "$1" = "add" ] || [ "$1" = "cd" ]; then
        local dir
        dir=$(command gwt "$@")
        local rc=$?
        if [ $rc -eq 0 ] && [ -n "$dir" ]; then
            builtin cd "$dir" || return 1
        fi
        return $rc
    else
        command gwt "$@"
    fi
}}
"#
    );
}
