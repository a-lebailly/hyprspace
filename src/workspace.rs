use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// Represents a workspace script found in ~/.config/hyprspace
#[derive(Debug)]
pub struct WorkspaceEntry {
    /// Short name (e.g. "backend" for "workspace-backend.sh")
    pub name_short: String,
    /// File name (e.g. "workspace-backend.sh")
    pub base_name: String,
    /// Full path to the script
    pub full_path: PathBuf,
    /// Parsed workspace number from the script (if found)
    pub workspace_num: Option<u32>,
}

/// Returns the path to the ~/.config/hyprspace directory
pub fn workspace_dir() -> PathBuf {
    let home = env::var("HOME").expect("HOME environment variable not set");
    let mut p = PathBuf::from(home);
    p.push(".config");
    p.push("hyprspace");
    p
}

/// Ensures that ~/.config/hyprspace exists, creating it if needed.
pub fn ensure_workspace_dir() -> io::Result<PathBuf> {
    let dir = workspace_dir();

    if !dir.exists() {
        println!(
            "Workspace directory {:?} does not exist, creating it...",
            dir.to_string_lossy()
        );
        fs::create_dir_all(&dir)?;
    }

    Ok(dir)
}

/// Try to parse `hyprctl dispatch workspace N` in the given script file.
fn parse_workspace_num(path: &Path) -> Option<u32> {
    let content = fs::read_to_string(path).ok()?;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // We look for a line that starts with "hyprctl dispatch workspace"
        if trimmed.starts_with("hyprctl dispatch workspace") {
            // Split by whitespace and parse the last token as a number
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if let Some(last) = parts.last() {
                if let Ok(num) = last.parse::<u32>() {
                    return Some(num);
                }
            }
        }
    }

    None
}

/// Lists all workspace-*.sh files in the given directory.
pub fn list_workspaces(dir: &Path) -> io::Result<Vec<WorkspaceEntry>> {
    let mut entries = Vec::new();

    if !dir.exists() {
        return Ok(entries);
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let file_name_os = match path.file_name() {
            Some(name) => name,
            None => continue,
        };

        let file_name = file_name_os.to_string_lossy();

        if !file_name.starts_with("workspace-") || !file_name.ends_with(".sh") {
            continue;
        }

        let no_ext = file_name.trim_end_matches(".sh");
        let short = no_ext.strip_prefix("workspace-").unwrap_or(no_ext);

        let workspace_num = parse_workspace_num(&path);

        entries.push(WorkspaceEntry {
            name_short: short.to_string(),
            base_name: file_name.to_string(),
            full_path: path,
            workspace_num,
        });
    }

    entries.sort_by(|a, b| a.base_name.cmp(&b.base_name));

    Ok(entries)
}

// Small helpers for a nicer interactive flow

fn clear_screen() {
    print!("\x1b[2J\x1b[H");
    let _ = io::stdout().flush();
}

fn prompt(label: &str) -> io::Result<String> {
    print!("{label}");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}

fn prompt_non_empty(label: &str) -> io::Result<String> {
    loop {
        let value = prompt(label)?;
        if !value.is_empty() {
            return Ok(value);
        }
        println!("  -> Value cannot be empty, try again.");
    }
}

fn prompt_yes_no(label: &str, default_no: bool) -> io::Result<bool> {
    let suffix = if default_no { " [y/N]: " } else { " [Y/n]: " };
    let full = format!("{label}{suffix}");
    let answer = prompt(&full)?.to_lowercase();

    if answer.is_empty() {
        return Ok(!default_no);
    }

    Ok(matches!(answer.as_str(), "y" | "yes"))
}

/// Create a new workspace script interactively (in normal terminal mode).
pub fn create_new_script(dir: &Path) -> io::Result<()> {
    clear_screen();

    // Some simple styling
    const BOLD: &str = "\x1b[1m";
    const CYAN: &str = "\x1b[36m";
    const RESET: &str = "\x1b[0m";

    println!(
        "{BOLD}{CYAN}Hyprspace · New workspace script{RESET}\n",
    );
    println!("Destination directory: {}", dir.to_string_lossy());
    println!("Follow the steps to configure your workspace layout.\n");

    // 1) Workspace number
    println!("{BOLD}Step 1/3 · Workspace target{RESET}");
    let workspace_num: u32 = loop {
        let value = prompt("Enter workspace number (e.g. 1, 2, 3): ")?;
        match value.trim().parse::<u32>() {
            Ok(n) if n > 0 => break n,
            _ => {
                println!("Invalid workspace number, please enter a positive integer.");
            }
        }
    };
    println!("Will dispatch to workspace {workspace_num}\n");

    // 2) Script short name
    println!("{BOLD}Step 2/3 · Script identity{RESET}");
    let short_name = prompt_non_empty("Enter script short name (e.g. 'backend', 'music', 'dashboard'): ")?;
    let file_name = format!("workspace-{}.sh", short_name);
    let mut path = dir.to_path_buf();
    path.push(&file_name);

    if path.exists() {
        println!(
            "File {} already exists, aborting.",
            path.to_string_lossy()
        );
        return Ok(());
    }

    println!("Script file will be: {}", path.to_string_lossy());
    println!();

    // 3) Build script content
    println!("{BOLD}Step 3/3 · Windows layout{RESET}");
    println!("You can now add one or more windows using rule_exec.");
    println!("For each window, you will choose size, position and command.\n");

    let mut content = String::new();

    // Header and workspace dispatch
    content.push_str("#!/bin/bash\n\n");
    content.push_str(&format!("hyprctl dispatch workspace {num}\n\n", num = workspace_num));

    // rule_exec helper
    content.push_str(
        r#"rule_exec() {
  local rules="$1"
  shift
  hyprctl dispatch exec "[$rules] $*"
}

"#,
    );

    let mut window_index = 1usize;

    // Add one or more rule_exec blocks
    loop {
        let add_window = prompt_yes_no("Add a window rule_exec?", true)?;
        if !add_window {
            break;
        }

        println!();
        println!(
            "{BOLD}Window #{idx}{RESET} – layout & command",
            idx = window_index
        );

        // Size: width / height
        let width = prompt_non_empty("  • width  (e.g. 10%): ")?;
        let height = prompt_non_empty("  • height (e.g. 15%): ")?;

        // Position: x / y
        let pos_x = prompt_non_empty("  • position X (e.g. 1%): ")?;
        let pos_y = prompt_non_empty("  • position Y (e.g. 8%): ")?;

        // Command to execute
        let command = prompt_non_empty(
            "command (e.g. kitty --hold zsh -c \"cava\" or firefox --new-window github.com): ",
        )?;

        // Append the rule_exec block
        content.push_str(&format!(
            "rule_exec \"workspace {num} silent; float; size {w} {h}; move {x} {y}\" \\\n  {cmd}\n\n",
            num = workspace_num,
            w = width,
            h = height,
            x = pos_x,
            y = pos_y,
            cmd = command
        ));

        println!("Window #{idx} added.", idx = window_index);
        window_index += 1;
    }

    if window_index == 1 {
        println!("\nNo windows were added. The script will only switch workspace.");
    }

    println!("\n{BOLD}Preview of the generated script:{RESET}\n");
    println!("----- {} -----", file_name);
    println!("{content}");
    println!("---------------------------\n");

    if !prompt_yes_no("Save this script?", false)? {
        println!("Aborted, script was not created.");
        return Ok(());
    }

    // Write file
    fs::write(&path, content)?;

    // Make it executable (chmod 755)
    let metadata = fs::metadata(&path)?;
    let mut perms = metadata.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms)?;

    println!("Created script: {}", path.to_string_lossy());
    Ok(())
}
