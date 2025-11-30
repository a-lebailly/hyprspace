mod workspace;
mod launcher;
mod tui;

use std::io;

use crate::launcher::launch_script;
use crate::tui::{run_tui, Action};
use crate::workspace::{create_new_script, ensure_workspace_dir, list_workspaces};

fn main() -> io::Result<()> {
    let dir = ensure_workspace_dir()?;
    let workspaces = list_workspaces(&dir)?;

    let (workspaces, action) = run_tui(workspaces)?;

    match action {
        Some(Action::Launch(idx)) => {
            if let Some(ws) = workspaces.get(idx) {
                launch_script(ws)?;
            }
        }
        Some(Action::CreateNew) => {
            // We are back in normal terminal mode here
            create_new_script(&dir)?;
        }
        None => {
            // User quit with q / Esc
        }
    }

    Ok(())
}
