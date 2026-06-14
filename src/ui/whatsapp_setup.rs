use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, WppPhase};

pub fn render(f: &mut Frame, app: &App) {
    let state = app.wpp_state.lock().unwrap();

    // ── Popup overlay ────────────────────────────────────────────────────────
    let area = popup_area(f.area());
    f.render_widget(Clear, area);

    let outer_block = Block::default()
        .title(" WhatsApp Setup ")
        .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let inner = outer_block.inner(area);
    f.render_widget(outer_block, area);

    // Inner layout: status bar + content + footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // status badge + log
            Constraint::Length(1), // separator
            Constraint::Min(0),    // content
            Constraint::Length(1), // separator
            Constraint::Length(1), // footer hints
        ])
        .split(inner);

    // ── Status line ─────────────────────────────────────────────────────────
    let (phase_label, phase_color) = phase_info(&state.phase);
    let status_line = Line::from(vec![
        Span::styled(
            format!(" {} ", phase_label),
            Style::default().fg(Color::Black).bg(phase_color).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(state.log.as_str(), Style::default().fg(Color::White)),
    ]);
    f.render_widget(Paragraph::new(status_line), chunks[0]);

    // separator line
    let sep = Paragraph::new("─".repeat(inner.width as usize))
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(sep, chunks[1]);

    // ── Content ─────────────────────────────────────────────────────────────
    let content_area = chunks[2];
    match &state.phase {
        WppPhase::ShowingQr => {
            if let Some(qr) = &state.qr_rendered {
                let block = Block::default()
                    .title(" WhatsApp → Dispositivos vinculados → Vincular ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green));
                f.render_widget(Paragraph::new(qr.as_str()).block(block), content_area);
            } else {
                f.render_widget(
                    Paragraph::new("Gerando QR code, aguarde...")
                        .style(Style::default().fg(Color::DarkGray)),
                    content_area,
                );
            }
        }

        WppPhase::AskPhone => {
            let parts = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(2), Constraint::Length(3), Constraint::Min(0)])
                .split(content_area);

            f.render_widget(
                Paragraph::new("WhatsApp conectado! Informe o número do tech lead:")
                    .style(Style::default().fg(Color::Green)),
                parts[0],
            );

            let cursor = "█";
            let phone_block = Block::default()
                .title(" Número com DDI — ex: 5511999999999 ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            f.render_widget(
                Paragraph::new(format!("{}{}", state.phone_input, cursor)).block(phone_block),
                parts[1],
            );
        }

        WppPhase::Done => {
            f.render_widget(
                Paragraph::new("\n ✅  WhatsApp configurado! Pressione [Esc] para fechar.")
                    .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                    .wrap(Wrap { trim: false }),
                content_area,
            );
        }

        WppPhase::Error(e) => {
            let msg = format!(" ✗  {}\n\n Verifique se o Docker está rodando e tente novamente.", e);
            f.render_widget(
                Paragraph::new(msg)
                    .style(Style::default().fg(Color::Red))
                    .wrap(Wrap { trim: false }),
                content_area,
            );
        }

        _ => {
            let steps = [
                (WppPhase::CheckingDocker,    "Verificando Docker"),
                (WppPhase::StartingContainer, "Iniciando container waha"),
                (WppPhase::WaitingApi,        "Aguardando API inicializar"),
                (WppPhase::CheckingInstance,  "Verificando sessão"),
                (WppPhase::CreatingInstance,  "Criando sessão"),
            ];

            let lines: Vec<Line> = steps
                .iter()
                .map(|(p, label)| {
                    let (icon, style) = if p == &state.phase {
                        ("▶", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                    } else if phase_order(p) < phase_order(&state.phase) {
                        ("✓", Style::default().fg(Color::Green))
                    } else {
                        ("·", Style::default().fg(Color::DarkGray))
                    };
                    Line::from(Span::styled(format!("  {}  {}", icon, label), style))
                })
                .collect();

            f.render_widget(Paragraph::new(lines), content_area);
        }
    }

    // ── Footer ──────────────────────────────────────────────────────────────
    let sep2 = Paragraph::new("─".repeat(inner.width as usize))
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(sep2, chunks[3]);

    let hint = match &state.phase {
        WppPhase::AskPhone => "[Enter] Salvar  [Esc] Cancelar",
        WppPhase::Done | WppPhase::Error(_) => "[Esc] Fechar",
        WppPhase::ShowingQr => "QR aberto no browser — [Esc] Cancelar",
        _ => "[Esc] Cancelar",
    };
    f.render_widget(
        Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)),
        chunks[4],
    );
}

/// Near-fullscreen popup: 98% wide, 96% tall.
fn popup_area(area: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(2),
            Constraint::Percentage(96),
            Constraint::Percentage(2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(1),
            Constraint::Percentage(98),
            Constraint::Percentage(1),
        ])
        .split(vert[1])[1]
}

fn phase_info(phase: &WppPhase) -> (&'static str, Color) {
    match phase {
        WppPhase::Idle             => ("INICIANDO",  Color::DarkGray),
        WppPhase::CheckingDocker   => ("DOCKER",     Color::Yellow),
        WppPhase::StartingContainer=> ("CONTAINER",  Color::Yellow),
        WppPhase::WaitingApi       => ("API",        Color::Yellow),
        WppPhase::CheckingInstance => ("SESSÃO",     Color::Yellow),
        WppPhase::CreatingInstance => ("CRIANDO",    Color::Yellow),
        WppPhase::ShowingQr        => ("QR CODE",    Color::Cyan),
        WppPhase::Connected        => ("CONECTADO",  Color::Green),
        WppPhase::AskPhone         => ("TELEFONE",   Color::Cyan),
        WppPhase::Done             => ("PRONTO",     Color::Green),
        WppPhase::Error(_)         => ("ERRO",       Color::Red),
    }
}

fn phase_order(phase: &WppPhase) -> u8 {
    match phase {
        WppPhase::Idle             => 0,
        WppPhase::CheckingDocker   => 1,
        WppPhase::StartingContainer=> 2,
        WppPhase::WaitingApi       => 3,
        WppPhase::CheckingInstance => 4,
        WppPhase::CreatingInstance => 5,
        WppPhase::ShowingQr        => 6,
        WppPhase::Connected        => 7,
        WppPhase::AskPhone         => 8,
        WppPhase::Done             => 9,
        WppPhase::Error(_)         => 99,
    }
}
