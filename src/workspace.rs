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

/// Create a new workspace script interactively (in normal terminal mode).
pub fn create_new_script(dir: &Path) -> io::Result<()> {
    println!(
        "Creating a new workspace script in {}",
        dir.to_string_lossy()
    );

    let mut input = String::new();

    // 1) First question: workspace number
    print!("Enter workspace number (e.g. 1, 2, 3): ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let workspace_num: u32 = match input.trim().parse() {
        Ok(n) if n > 0 => n,
        _ => {
            println!("Invalid workspace number, aborting.");
            return Ok(());
        }
    };

    // 2) Second question: script short name
    print!("Enter script short name (e.g. 'backend', 'music', 'dashboard'): ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let short_name = input.trim();
    if short_name.is_empty() {
        println!("Empty script name, aborting.");
        return Ok(());
    }

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

    // 3) Build script content
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

    // 4) Add one or more rule_exec blocks
    loop {
        input.clear();
        print!("Add a window rule_exec? [y/N]: ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
        let answer = input.trim().to_lowercase();
        if answer != "y" && answer != "yes" {
            break;
        }

        // Size: width / height
        input.clear();
        print!("  • width (e.g. 10%): ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
        let width = input.trim().to_string();
        if width.is_empty() {
            println!("    Empty width, skipping this window.");
            continue;
        }

        input.clear();
        print!("  • height (e.g. 15%): ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
        let height = input.trim().to_string();
        if height.is_empty() {
            println!("    Empty height, skipping this window.");
            continue;
        }

        // Position: x / y
        input.clear();
        print!("  • position X (e.g. 1%): ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
        let pos_x = input.trim().to_string();
        if pos_x.is_empty() {
            println!("    Empty X position, skipping this window.");
            continue;
        }

        input.clear();
        print!("  • position Y (e.g. 8%): ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
        let pos_y = input.trim().to_string();
        if pos_y.is_empty() {
            println!("    Empty Y position, skipping this window.");
            continue;
        }

        // Command to execute
        input.clear();
        print!("  • command to execute (e.g. kitty --hold zsh -c \"clock\"): ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
        let command = input.trim().to_string();
        if command.is_empty() {
            println!("    Empty command, skipping this window.");
            continue;
        }

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
