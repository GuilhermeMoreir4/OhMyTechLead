use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{
    App, CUSTOM_CAT_ADD_IDX, DISCORD_TOGGLE_IDX, SETTINGS_COUNT, TOGGLE_IDX,
    S_DISCORD_TOKEN, S_DISCORD_USER,
    S_WPP_URL, S_WPP_KEY, S_WPP_INSTANCE, S_WPP_PHONE,
};

struct FieldMeta {
    label: &'static str,
    masked: bool,
}

const FIELDS: [FieldMeta; SETTINGS_COUNT] = [
    FieldMeta { label: "Discord — Bot Token",         masked: true  },
    FieldMeta { label: "Discord — User ID Tech Lead", masked: false },
    FieldMeta { label: "WhatsApp — URL (waha)",       masked: false },
    FieldMeta { label: "WhatsApp — API Key",          masked: true  },
    FieldMeta { label: "WhatsApp — Sessão",           masked: false },
    FieldMeta { label: "WhatsApp — Telefone (5511…)", masked: false },
    FieldMeta { label: "Horário de Envio (HH:MM)",    masked: false },
];

// Visual layout:
//  0   Discord Token
//  1   Discord User ID
//  2   Discord toggle
//  3   WhatsApp toggle
//  4   WPP URL
//  5   WPP Key
//  6   WPP Sessão
//  7   WPP Telefone
//  8   Horário
//  9   "+ Adicionar Coluna"  (CUSTOM_CAT_ADD_IDX)
//  10+ existing custom categories

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();
    let custom_count = app.custom_categories.len();
    // 9 base rows + 1 "add" button + N custom categories
    let visual_rows = SETTINGS_COUNT + 2 + 1 + custom_count;

    let mut constraints = vec![Constraint::Length(3)]; // header
    for _ in 0..visual_rows {
        constraints.push(Constraint::Length(3));
    }
    constraints.push(Constraint::Min(0));
    constraints.push(Constraint::Length(3)); // footer

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let header = Paragraph::new(
        " Configurações  [↑/↓] Navegar  [Enter] Editar  [Space] Toggle  [Ctrl+S] Salvar  [Esc] Voltar",
    )
    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    for visual_row in 0..visual_rows {
        let chunk = chunks[visual_row + 1];

        // Discord toggle
        if visual_row == 2 {
            render_toggle(f, chunk, "Discord — Ativado", app.discord_enabled, app.settings_field == DISCORD_TOGGLE_IDX);
            continue;
        }

        // WhatsApp toggle
        if visual_row == 3 {
            render_toggle(f, chunk, "WhatsApp — Ativado", app.wpp_enabled, app.settings_field == TOGGLE_IDX);
            continue;
        }

        // "+ Adicionar Coluna" button
        if visual_row == CUSTOM_CAT_ADD_IDX {
            let is_focused = app.settings_field == CUSTOM_CAT_ADD_IDX;
            let border_style = if is_focused {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let block = Block::default()
                .title(" ── Colunas Extras ── ")
                .borders(Borders::ALL)
                .border_style(border_style);
            let hint = if is_focused { "[Enter] Adicionar nova coluna" } else { "+ Adicionar Coluna" };
            f.render_widget(
                Paragraph::new(hint).style(Style::default().fg(Color::Green)).block(block),
                chunk,
            );
            continue;
        }

        // Existing custom categories
        if visual_row > CUSTOM_CAT_ADD_IDX {
            let cat_idx = visual_row - CUSTOM_CAT_ADD_IDX - 1;
            if cat_idx < custom_count {
                let cc = &app.custom_categories[cat_idx];
                let field_idx = CUSTOM_CAT_ADD_IDX + 1 + cat_idx;
                let is_focused = app.settings_field == field_idx;
                let border_style = if is_focused {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                let block = Block::default()
                    .title(format!(" Coluna Extra #{} ", cat_idx + 1))
                    .borders(Borders::ALL)
                    .border_style(border_style);
                let label = format!("{} {}  {}", cc.icon, cc.name, if is_focused { "  [D] Remover" } else { "" });
                f.render_widget(Paragraph::new(label).block(block), chunk);
            }
            continue;
        }

        // Normal text fields — map visual_row to field index
        // visual_row < 2 → field_idx = visual_row
        // visual_row in 4..8 → field_idx = visual_row - 2
        let field_idx = if visual_row < 2 { visual_row } else { visual_row - 2 };
        let meta = &FIELDS[field_idx];

        let is_focused = app.settings_field == field_idx;
        let is_editing = is_focused && app.settings_editing;

        let discord_field = matches!(field_idx, S_DISCORD_TOKEN | S_DISCORD_USER);
        let wpp_field = matches!(field_idx, S_WPP_URL | S_WPP_KEY | S_WPP_INSTANCE | S_WPP_PHONE);
        let dimmed = (discord_field && !app.discord_enabled) || (wpp_field && !app.wpp_enabled);

        let border_style = if dimmed {
            Style::default().fg(Color::DarkGray)
        } else if is_editing {
            Style::default().fg(Color::Green)
        } else if is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let value = &app.settings_inputs[field_idx];
        let cursor = if is_editing { "█" } else { "" };
        let display = if meta.masked && !value.is_empty() && !is_editing {
            format!("{}***", &value[..value.len().min(10)])
        } else {
            format!("{}{}", value, cursor)
        };

        let text_style = if dimmed {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };

        let block = Block::default()
            .title(format!(" {} ", meta.label))
            .borders(Borders::ALL)
            .border_style(border_style);

        f.render_widget(Paragraph::new(Span::styled(display, text_style)).block(block), chunk);
    }

    let hint = if app.settings_editing {
        "[Enter/Esc] Confirmar"
    } else {
        "[Ctrl+S] Salvar  [Esc] Voltar"
    };
    let footer = Paragraph::new(hint)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, *chunks.last().unwrap());
}

fn render_toggle(f: &mut Frame, area: ratatui::layout::Rect, title: &str, enabled: bool, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let status = if enabled {
        Span::styled(" ● Ativado", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(" ○ Desativado", Style::default().fg(Color::DarkGray))
    };
    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(border_style);
    let line = Line::from(vec![status, Span::raw("   [Space] alternar")]);
    f.render_widget(Paragraph::new(line).block(block), area);
}
