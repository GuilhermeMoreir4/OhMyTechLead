use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::report::generate_report;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let report = generate_report(app.date, &app.tasks);

    let block = Block::default()
        .title(" Preview do Relatório ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let preview = Paragraph::new(report)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(preview, chunks[0]);

    let footer = Paragraph::new("[S] Enviar agora  [Esc] Voltar")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[1]);
}
