use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

use crate::workspace::WorkspaceEntry;

/// What the user chose in the TUI
#[derive(Debug, Clone, Copy)]
pub enum Action {
    Launch(usize),
    CreateNew,
}

/// Application state for the TUI
struct App {
    workspaces: Vec<WorkspaceEntry>,
    selected: usize,
    action: Option<Action>,
}

impl App {
    fn new(workspaces: Vec<WorkspaceEntry>) -> Self {
        Self {
            workspaces,
            selected: 0,
            action: None,
        }
    }

    fn total_items(&self) -> usize {
        // all workspaces + 1 extra item for "Create new..."
        self.workspaces.len() + 1
    }

    fn next(&mut self) {
        let total = self.total_items();
        if total == 0 {
            return;
        }
        self.selected = (self.selected + 1) % total;
    }

    fn previous(&mut self) {
        let total = self.total_items();
        if total == 0 {
            return;
        }
        if self.selected == 0 {
            self.selected = total - 1;
        } else {
            self.selected -= 1;
        }
    }
}

/// Draw the UI for the current app state
fn ui(f: &mut Frame, app: &App) {
    let area = f.area();

    let title = format!(
        "Hyprspace • {} configuration(s) found",
        app.workspaces.len()
    );

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL);

    // Build list items: all workspaces + one "Create new" entry
    let mut items: Vec<ListItem> = app
        .workspaces
        .iter()
        .enumerate()
        .map(|(idx, ws)| {
            let ws_info = match ws.workspace_num {
                Some(num) => format!("[ws {}]", num),
                None => "[ws ?]".to_string(),
            };

            let text = format!(
                "{}. {} {} ({})",
                idx + 1,
                ws_info,
                ws.name_short,
                ws.base_name
            );
            ListItem::new(text)
        })
        .collect();

    let create_label = "➕ Create new workspace script…";
    items.push(ListItem::new(create_label));

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("➤ ");

    let mut state = ListState::default();
    if app.total_items() > 0 {
        state.select(Some(app.selected));
    }

    f.render_stateful_widget(list, area, &mut state);
}

/// Run the TUI and return the selected action (launch or create).
pub fn run_tui(
    workspaces: Vec<WorkspaceEntry>,
) -> io::Result<(Vec<WorkspaceEntry>, Option<Action>)> {
    let mut app = App::new(workspaces);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.action = None;
                        break;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.next();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.previous();
                    }
                    KeyCode::Enter => {
                        let ws_len = app.workspaces.len();
                        if app.selected < ws_len {
                            app.action = Some(Action::Launch(app.selected));
                        } else {
                            app.action = Some(Action::CreateNew);
                        }
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok((app.workspaces, app.action))
}
