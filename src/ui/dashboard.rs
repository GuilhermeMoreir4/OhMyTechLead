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

    // outer layout: header / body / footer
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
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    let cells = [top[0], top[1], bottom[0], bottom[1]];
    let categories = Category::all();

    for (i, (cat, cell)) in categories.iter().zip(cells.iter()).enumerate() {
        render_category_panel(f, app, cat, *cell, i == app.selected_category);
    }
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
            ListItem::new(Line::from(vec![Span::styled(
                format!("{}{}", prefix, t.description),
                style,
            )]))
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
