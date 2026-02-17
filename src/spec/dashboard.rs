use std::collections::HashSet;
use std::io;
use std::io::IsTerminal;
use std::sync::mpsc;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use ratatui::prelude::*;
use ratatui::widgets::*;

use super::specs_dir;
use super::summary::{SpecStatus, SpecSummary, load_all_summaries};

// ---------------------------------------------------------------------------
// Display model
// ---------------------------------------------------------------------------

#[derive(Clone)]
enum DisplayItem {
    GroupHeader {
        name: String,
        checked: u32,
        total: u32,
    },
    Spec(usize), // index into App::specs
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

enum Mode {
    List,
    Detail,
}

struct DetailState {
    spec_index: usize,
    collapsed: HashSet<usize>, // indices of collapsed top-level tasks
    selected: usize,           // index into visible detail rows
}

struct App {
    specs: Vec<SpecSummary>,
    display_items: Vec<DisplayItem>,
    /// Indices into display_items that are selectable (Spec rows only).
    selectable: Vec<usize>,
    selected: usize, // index into selectable
    mode: Mode,
    detail: DetailState,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let mut app = App {
            specs: Vec::new(),
            display_items: Vec::new(),
            selectable: Vec::new(),
            selected: 0,
            mode: Mode::List,
            detail: DetailState {
                spec_index: 0,
                collapsed: HashSet::new(),
                selected: 0,
            },
            should_quit: false,
        };
        app.reload();
        app
    }

    fn reload(&mut self) {
        self.specs = load_all_summaries().unwrap_or_default();
        self.build_display_items();

        // Clamp list selection
        if !self.selectable.is_empty() {
            self.selected = self.selected.min(self.selectable.len() - 1);
        } else {
            self.selected = 0;
        }
    }

    fn build_display_items(&mut self) {
        self.display_items.clear();
        self.selectable.clear();

        // Group specs: ungrouped first, then by group name.
        // Within each status tier, specs are already sorted by group then name.
        let mut current_group: Option<&str> = None;
        let mut current_status: Option<&SpecStatus> = None;

        for (idx, spec) in self.specs.iter().enumerate() {
            let group_changed = spec.group.as_deref() != current_group;
            let status_changed = current_status != Some(&spec.status);

            // Emit group header when entering a new group
            if group_changed {
                if let Some(ref g) = spec.group {
                    // Compute aggregate for this group
                    let (gc, gt) = self
                        .specs
                        .iter()
                        .filter(|s| s.group.as_deref() == Some(g.as_str()))
                        .fold((0u32, 0u32), |(c, t), s| (c + s.checked, t + s.total));
                    self.display_items.push(DisplayItem::GroupHeader {
                        name: g.clone(),
                        checked: gc,
                        total: gt,
                    });
                } else if status_changed && current_group.is_some() {
                    // Transitioning from grouped back to ungrouped in a new status tier
                    // (unlikely given sort order, but handle gracefully)
                }
            }
            current_group = spec.group.as_deref();
            current_status = Some(&spec.status);

            self.selectable.push(self.display_items.len());
            self.display_items.push(DisplayItem::Spec(idx));
        }
    }

    /// The spec index pointed to by the current list selection.
    fn selected_spec_index(&self) -> Option<usize> {
        let flat = *self.selectable.get(self.selected)?;
        match &self.display_items[flat] {
            DisplayItem::Spec(idx) => Some(*idx),
            _ => None,
        }
    }

    /// Build flat list of visible detail rows for the detail view.
    fn detail_rows(&self) -> Vec<DetailRow> {
        let spec = &self.specs[self.detail.spec_index];
        let mut rows = Vec::new();
        for (i, task) in spec.tasks.iter().enumerate() {
            let expanded = !self.detail.collapsed.contains(&i);
            rows.push(DetailRow::TopLevel { index: i, expanded });
            if expanded {
                for j in 0..task.children.len() {
                    rows.push(DetailRow::SubTask {
                        parent: i,
                        child: j,
                    });
                }
            }
        }
        rows
    }
}

#[derive(Clone)]
enum DetailRow {
    TopLevel { index: usize, expanded: bool },
    SubTask { parent: usize, child: usize },
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run() -> Result<(), String> {
    if !io::stdout().is_terminal() {
        return Err("Dashboard requires an interactive terminal".into());
    }

    enable_raw_mode().map_err(|e| e.to_string())?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| e.to_string())?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| e.to_string())?;

    // File watcher
    let (tx, rx) = mpsc::channel();
    let mut _watcher = setup_watcher(tx);

    let mut app = App::new();
    let result = main_loop(&mut terminal, &mut app, &rx);

    // Restore terminal
    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    result
}

fn setup_watcher(tx: mpsc::Sender<notify::Result<notify::Event>>) -> Option<RecommendedWatcher> {
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            tx.send(res).ok();
        },
        Config::default(),
    )
    .ok()?;
    let dir = specs_dir();
    if dir.exists() {
        watcher.watch(dir.as_ref(), RecursiveMode::Recursive).ok()?;
    }
    Some(watcher)
}

// ---------------------------------------------------------------------------
// Main loop
// ---------------------------------------------------------------------------

fn main_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    fs_rx: &mpsc::Receiver<notify::Result<notify::Event>>,
) -> Result<(), String> {
    loop {
        terminal
            .draw(|frame| ui(frame, app))
            .map_err(|e| e.to_string())?;

        // Drain file system events and reload once
        let mut needs_reload = false;
        while fs_rx.try_recv().is_ok() {
            needs_reload = true;
        }
        if needs_reload {
            app.reload();
        }

        if event::poll(Duration::from_millis(250)).map_err(|e| e.to_string())?
            && let Event::Key(key) = event::read().map_err(|e| e.to_string())?
                && key.kind == KeyEventKind::Press {
                    match app.mode {
                        Mode::List => handle_list_key(app, key.code),
                        Mode::Detail => handle_detail_key(app, key.code),
                    }
                }

        if app.should_quit {
            return Ok(());
        }
    }
}

// ---------------------------------------------------------------------------
// Key handlers
// ---------------------------------------------------------------------------

fn handle_list_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected > 0 {
                app.selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !app.selectable.is_empty() && app.selected < app.selectable.len() - 1 {
                app.selected += 1;
            }
        }
        KeyCode::Enter => {
            if let Some(idx) = app.selected_spec_index() {
                app.detail = DetailState {
                    spec_index: idx,
                    collapsed: HashSet::new(),
                    selected: 0,
                };
                app.mode = Mode::Detail;
            }
        }
        _ => {}
    }
}

fn handle_detail_key(app: &mut App, code: KeyCode) {
    let row_count = app.detail_rows().len();
    match code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Esc => app.mode = Mode::List,
        KeyCode::Up | KeyCode::Char('k') => {
            if app.detail.selected > 0 {
                app.detail.selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if row_count > 0 && app.detail.selected < row_count - 1 {
                app.detail.selected += 1;
            }
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            let rows = app.detail_rows();
            if let Some(DetailRow::TopLevel { index, .. }) = rows.get(app.detail.selected) {
                let idx = *index;
                if app.detail.collapsed.contains(&idx) {
                    app.detail.collapsed.remove(&idx);
                } else {
                    app.detail.collapsed.insert(idx);
                }
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn ui(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Min(0),    // content
            Constraint::Length(1), // help
        ])
        .split(area);

    // Title bar
    let title = match app.mode {
        Mode::List => Line::from(vec![
            Span::styled(
                " tinyspec",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" dashboard"),
        ]),
        Mode::Detail => {
            let spec = &app.specs[app.detail.spec_index];
            Line::from(vec![
                Span::styled(
                    format!(" {}", spec.name),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" — Implementation Plan"),
            ])
        }
    };
    frame.render_widget(
        Paragraph::new(title).style(Style::default().bg(Color::DarkGray)),
        chunks[0],
    );

    // Content
    match app.mode {
        Mode::List => render_list(frame, app, chunks[1]),
        Mode::Detail => render_detail(frame, app, chunks[1]),
    }

    // Help bar
    let help = match app.mode {
        Mode::List => " ↑↓/jk navigate  Enter detail  q quit",
        Mode::Detail => " ↑↓/jk navigate  Enter toggle  Esc back  q quit",
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            help,
            Style::default().fg(Color::DarkGray),
        ))),
        chunks[2],
    );
}

fn render_list(frame: &mut Frame, app: &mut App, area: Rect) {
    if app.specs.is_empty() {
        let msg = Paragraph::new("\n  No specs found. Create one with: tinyspec new <name>")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, area);
        return;
    }

    let bar_width = 10usize;

    let items: Vec<ListItem> = app
        .display_items
        .iter()
        .map(|item| match item {
            DisplayItem::GroupHeader {
                name,
                checked,
                total,
            } => {
                let pct = if *total > 0 {
                    *checked as f64 / *total as f64 * 100.0
                } else {
                    0.0
                };
                ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("{name}/"),
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!("  {pct:.0}%"), Style::default().fg(Color::DarkGray)),
                ]))
            }
            DisplayItem::Spec(idx) => {
                let spec = &app.specs[*idx];
                let (icon, icon_color) = match spec.status {
                    SpecStatus::InProgress => ("●", Color::Yellow),
                    SpecStatus::Pending => ("○", Color::DarkGray),
                    SpecStatus::Completed => ("✓", Color::Green),
                };

                let filled = if spec.total > 0 {
                    (spec.checked as f64 / spec.total as f64 * bar_width as f64).round() as usize
                } else {
                    0
                };
                let empty = bar_width - filled;

                let bar_color = match spec.status {
                    SpecStatus::Completed => Color::Green,
                    SpecStatus::InProgress => Color::Yellow,
                    SpecStatus::Pending => Color::DarkGray,
                };

                ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(icon, Style::default().fg(icon_color)),
                    Span::raw(" "),
                    Span::raw(format!("{:<28}", spec.name)),
                    Span::styled("█".repeat(filled), Style::default().fg(bar_color)),
                    Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
                    Span::raw(format!("  {}/{}", spec.checked, spec.total)),
                ]))
            }
        })
        .collect();

    // Compute the flat index for ListState from our logical selection
    let flat_index = app.selectable.get(app.selected).copied();
    let mut list_state = ListState::default().with_selected(flat_index);

    let list = List::new(items).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_detail(frame: &mut Frame, app: &mut App, area: Rect) {
    let spec = &app.specs[app.detail.spec_index];
    let rows = app.detail_rows();

    let items: Vec<ListItem> = rows
        .iter()
        .map(|row| match row {
            DetailRow::TopLevel { index, expanded } => {
                let task = &spec.tasks[*index];
                let arrow = if task.children.is_empty() {
                    " "
                } else if *expanded {
                    "▼"
                } else {
                    "▶"
                };
                let check = if task.checked { "✓" } else { "☐" };
                let check_color = if task.checked {
                    Color::Green
                } else {
                    Color::default()
                };

                let child_progress = if !task.children.is_empty() {
                    let done = task.children.iter().filter(|c| c.checked).count();
                    format!("  [{}/{}]", done, task.children.len())
                } else {
                    String::new()
                };

                ListItem::new(Line::from(vec![
                    Span::raw(format!("  {arrow} ")),
                    Span::styled(check, Style::default().fg(check_color)),
                    Span::raw(format!(" {}: {}", task.id, task.description)),
                    Span::styled(child_progress, Style::default().fg(Color::DarkGray)),
                ]))
            }
            DetailRow::SubTask { parent, child } => {
                let task = &spec.tasks[*parent].children[*child];
                let check = if task.checked { "✓" } else { "☐" };
                let check_color = if task.checked {
                    Color::Green
                } else {
                    Color::default()
                };

                ListItem::new(Line::from(vec![
                    Span::raw("      "),
                    Span::styled(check, Style::default().fg(check_color)),
                    Span::raw(format!(" {}: {}", task.id, task.description)),
                ]))
            }
        })
        .collect();

    let mut list_state = ListState::default().with_selected(Some(app.detail.selected));

    let list = List::new(items).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    frame.render_stateful_widget(list, area, &mut list_state);
}
