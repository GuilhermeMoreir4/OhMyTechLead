pub mod dashboard;
pub mod add_task;
pub mod preview;
pub mod settings;
pub mod whatsapp_setup;

use ratatui::Frame;
use crate::app::{App, Screen};

pub fn render(f: &mut Frame, app: &App) {
    match app.screen {
        Screen::Dashboard => dashboard::render(f, app),
        Screen::AddTask => {
            dashboard::render(f, app);
            add_task::render(f, app);
        }
        Screen::ConfirmSend => {
            dashboard::render(f, app);
            render_confirm(f);
        }
        Screen::PreviewReport => preview::render(f, app),
        Screen::Settings => settings::render(f, app),
        Screen::WhatsappSetup => {
            dashboard::render(f, app);
            whatsapp_setup::render(f, app);
        }
        Screen::AddCustomCategory => {
            settings::render(f, app);
            render_add_custom_category(f, app);
        }
    }
}

fn render_add_custom_category(f: &mut Frame, app: &App) {
    use ratatui::{
        style::{Color, Style},
        widgets::{Block, Borders, Clear, Paragraph},
    };
    let area = centered_rect(50, 30, f.area());
    f.render_widget(Clear, area);

    let (title, value, hint) = if app.custom_cat_step == 0 {
        (" Nova Coluna — Nome ", &app.custom_cat_name, "[Enter] Próximo  [Esc] Cancelar")
    } else {
        (" Nova Coluna — Emoji ", &app.custom_cat_icon, "[Enter] Confirmar  [Esc] Voltar")
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(area);
    f.render_widget(block, area);

    use ratatui::layout::{Constraint, Direction, Layout};
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    f.render_widget(
        Paragraph::new(format!("{}█", value)).style(Style::default().fg(Color::White)),
        chunks[0],
    );
    f.render_widget(
        Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)),
        chunks[1],
    );
}

fn render_confirm(f: &mut Frame) {
    use ratatui::{
        style::{Color, Style},
        widgets::{Block, Borders, Clear, Paragraph},
    };
    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);
    let block = Block::default()
        .title(" Confirmar envio ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let text = Paragraph::new("Enviar relatório de hoje para o tech lead?\n\n[Enter] Confirmar   [Esc] Cancelar")
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(text, area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    use ratatui::layout::{Constraint, Direction, Layout};
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
