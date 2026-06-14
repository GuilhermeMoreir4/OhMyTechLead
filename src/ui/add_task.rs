use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{AddTaskStep, App};
use crate::storage::Category;
use crate::ui::centered_rect;

pub fn render(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Nova Task ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(8), Constraint::Length(3), Constraint::Min(0)])
        .split(block.inner(area));

    f.render_widget(block, area);

    // Category selector
    let categories = Category::all();
    let items: Vec<ListItem> = categories
        .iter()
        .enumerate()
        .map(|(i, cat)| {
            let is_sel = i == app.add_category;
            let style = if is_sel {
                Style::default().fg(Color::Black).bg(Color::Green)
            } else {
                Style::default()
            };
            let prefix = if is_sel { "> " } else { "  " };
            ListItem::new(Line::from(vec![Span::styled(
                format!("{}{} {}", prefix, cat.icon(), cat.label()),
                style,
            )]))
        })
        .collect();

    let cat_block = Block::default()
        .title(" Categoria ")
        .borders(Borders::ALL)
        .border_style(if app.add_step == AddTaskStep::SelectCategory {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let list = List::new(items).block(cat_block);
    f.render_widget(list, inner[0]);

    // Description input
    let desc_style = if app.add_step == AddTaskStep::EnterDescription {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let cursor = if app.add_step == AddTaskStep::EnterDescription {
        "█"
    } else {
        ""
    };

    let desc_block = Block::default()
        .title(" Descrição ")
        .borders(Borders::ALL)
        .border_style(desc_style);

    let desc = Paragraph::new(format!("{}{}", app.add_input, cursor)).block(desc_block);
    f.render_widget(desc, inner[1]);

    let hint = match app.add_step {
        AddTaskStep::SelectCategory => "[↑/↓] Selecionar  [Enter] Confirmar  [Esc] Cancelar",
        AddTaskStep::EnterDescription => "[Enter] Salvar  [Esc] Voltar",
    };
    let hint_widget = Paragraph::new(hint).style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC));
    f.render_widget(hint_widget, inner[2]);
}
