use std::io;
use std::process::{Command, Stdio};

use crate::workspace::WorkspaceEntry;

/// Launch the selected script (after TUI has been restored)
pub fn launch_script(ws: &WorkspaceEntry) -> io::Result<()> {
    println!("Launching: {}", ws.base_name);
    println!("Path: {}", ws.full_path.to_string_lossy());
    println!();

    let mut child = Command::new(ws.full_path.clone())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let _ = child.wait();

    Ok(())
}
