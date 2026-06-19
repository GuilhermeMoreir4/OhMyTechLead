use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::App;
use crate::report::format_date;
use crate::storage::Category;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    render_header(f, app, chunks[0]);
    render_body(f, app, chunks[1]);
    render_footer(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let title = format!(
        " oh-my-tech-lead  │  {} ",
        format_date(app.date)
    );
    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, area);
}

fn render_body(f: &mut Frame, app: &App, area: Rect) {
    let categories = app.all_categories();
    let n = categories.len();
    if n == 0 { return; }

    let row_count = (n + 1) / 2;
    let row_constraints: Vec<Constraint> = (0..row_count)
        .map(|_| Constraint::Ratio(1, row_count as u32))
        .collect();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(area);

    for (row_idx, row_area) in rows.iter().enumerate() {
        let left_idx = row_idx * 2;
        let right_idx = left_idx + 1;
        let has_right = right_idx < n;

        let cols = if has_right {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(*row_area)
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)])
                .split(*row_area)
        };

        render_category_panel(f, app, &categories[left_idx], cols[0], left_idx == app.selected_category);

        if has_right {
            render_category_panel(f, app, &categories[right_idx], cols[1], right_idx == app.selected_category);
        }
    }
}

/// Wraps `text` into lines of at most `max_width` characters (by char count).
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 || text.is_empty() {
        return vec![text.to_string()];
    }
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.chars().count() + 1 + word.chars().count() <= max_width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current.clone());
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn render_category_panel(f: &mut Frame, app: &App, cat: &Category, area: Rect, is_selected: bool) {
    let tasks = app.tasks_for_category(cat);
    let title = format!(" {} {} ({}) ", cat.icon(), cat.label(), tasks.len());

    let border_style = if is_selected {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    // Available width: panel width - 2 (borders) - 2 (prefix "  " or "> ")
    let available_width = (area.width as usize).saturating_sub(4);

    let items: Vec<ListItem> = tasks
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let is_item_selected = is_selected && i == app.selected_task;
            let style = if is_item_selected {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default()
            };
            let prefix = if is_item_selected { "> " } else { "  " };

            let wrapped = wrap_text(&t.description, available_width.saturating_sub(2));
            let lines: Vec<Line> = wrapped
                .iter()
                .enumerate()
                .map(|(li, text)| {
                    let display = if li == 0 {
                        format!("{}{}", prefix, text)
                    } else {
                        format!("  {}", text)
                    };
                    Line::from(vec![Span::styled(display, style)])
                })
                .collect();
            ListItem::new(lines)
        })
        .collect();

    if items.is_empty() {
        let empty = Paragraph::new("  (vazio)")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        f.render_widget(empty, area);
    } else {
        let mut state = ListState::default();
        if is_selected {
            state.select(Some(app.selected_task));
        }
        let list = List::new(items).block(block);
        f.render_stateful_widget(list, area, &mut state);
    }
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let status = app
        .status_message
        .as_deref()
        .unwrap_or("[N] Nova  [D] Deletar  [P] Preview  [S] Enviar  [W] WhatsApp  [C] Config  [Tab] Categoria  [↑/↓] Task  [Q] Sair");
    let footer = Paragraph::new(status)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, area);
}
