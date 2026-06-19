use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::app::{
    AddTaskStep, App, Screen, WppPhase,
    CUSTOM_CAT_ADD_IDX, DISCORD_TOGGLE_IDX, SETTINGS_COUNT, TOGGLE_IDX,
};
use crate::ui;

pub fn run_tui() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new()?;

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                app.status_message = None;
                handle_key(&mut app, key.code, key.modifiers)?;
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn handle_key(app: &mut App, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
    match app.screen.clone() {
        Screen::Dashboard          => handle_dashboard(app, key),
        Screen::AddTask            => handle_add_task(app, key),
        Screen::PreviewReport      => handle_preview(app, key)?,
        Screen::ConfirmSend        => handle_confirm_send(app, key)?,
        Screen::Settings           => handle_settings(app, key, modifiers)?,
        Screen::WhatsappSetup      => handle_wpp_setup(app, key)?,
        Screen::AddCustomCategory  => handle_add_custom_category(app, key)?,
    }
    Ok(())
}

fn handle_dashboard(app: &mut App, key: KeyCode) {
    let cat_count = app.all_categories().len();
    match key {
        KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
        KeyCode::Char('n') | KeyCode::Char('N') => {
            app.add_step = AddTaskStep::SelectCategory;
            app.add_category = app.selected_category;
            app.add_input.clear();
            app.screen = Screen::AddTask;
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            app.delete_selected_task();
        }
        KeyCode::Char('p') | KeyCode::Char('P') => {
            app.screen = Screen::PreviewReport;
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            app.screen = Screen::ConfirmSend;
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            app.screen = Screen::Settings;
        }
        KeyCode::Char('w') | KeyCode::Char('W') => {
            enter_wpp_setup(app);
        }
        KeyCode::Tab | KeyCode::Right => {
            app.selected_category = (app.selected_category + 1) % cat_count;
            app.selected_task = 0;
        }
        KeyCode::BackTab | KeyCode::Left => {
            app.selected_category = (app.selected_category + cat_count - 1) % cat_count;
            app.selected_task = 0;
        }
        KeyCode::Up => {
            if app.selected_task > 0 {
                app.selected_task -= 1;
            }
        }
        KeyCode::Down => {
            let cat = app.selected_category_enum();
            let count = app.tasks_for_category(&cat).len();
            if app.selected_task + 1 < count {
                app.selected_task += 1;
            }
        }
        _ => {}
    }
}

fn enter_wpp_setup(app: &mut App) {
    let (state, cancel) = app.start_wpp_setup();
    app.screen = Screen::WhatsappSetup;

    tokio::runtime::Handle::current().spawn(async move {
        crate::wpp_setup::run(state, cancel).await;
    });
}

fn handle_add_task(app: &mut App, key: KeyCode) {
    let cat_count = app.all_categories().len();
    match app.add_step {
        AddTaskStep::SelectCategory => match key {
            KeyCode::Esc => app.screen = Screen::Dashboard,
            KeyCode::Up => {
                if app.add_category > 0 { app.add_category -= 1; }
            }
            KeyCode::Down => {
                if app.add_category + 1 < cat_count { app.add_category += 1; }
            }
            KeyCode::Enter => {
                app.add_step = AddTaskStep::EnterDescription;
            }
            _ => {}
        },
        AddTaskStep::EnterDescription => match key {
            KeyCode::Esc => {
                app.add_step = AddTaskStep::SelectCategory;
            }
            KeyCode::Enter => {
                app.add_task();
                app.clamp_selected_task();
            }
            KeyCode::Backspace => { app.add_input.pop(); }
            KeyCode::Char(c) => { app.add_input.push(c); }
            _ => {}
        },
    }
}

fn handle_preview(app: &mut App, key: KeyCode) -> Result<()> {
    match key {
        KeyCode::Esc => app.screen = Screen::Dashboard,
        KeyCode::Char('s') | KeyCode::Char('S') => {
            app.screen = Screen::ConfirmSend;
        }
        _ => {}
    }
    Ok(())
}

fn handle_confirm_send(app: &mut App, key: KeyCode) -> Result<()> {
    match key {
        KeyCode::Esc => app.screen = Screen::Dashboard,
        KeyCode::Enter => {
            app.screen = Screen::Dashboard;
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(crate::scheduler::send_now())
            });
            match result {
                Ok(()) => app.set_status("Relatório enviado com sucesso!"),
                Err(e) => app.set_status(&format!("Erro: {e}")),
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_settings(app: &mut App, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
    let max_idx = CUSTOM_CAT_ADD_IDX + app.custom_categories.len();

    if app.settings_editing {
        match key {
            KeyCode::Esc | KeyCode::Enter => {
                app.settings_editing = false;
            }
            KeyCode::Backspace => {
                app.settings_inputs[app.settings_field].pop();
            }
            KeyCode::Char(c) => {
                app.settings_inputs[app.settings_field].push(c);
            }
            _ => {}
        }
    } else {
        match key {
            KeyCode::Esc => app.screen = Screen::Dashboard,
            KeyCode::Up => {
                if app.settings_field > 0 { app.settings_field -= 1; }
            }
            KeyCode::Down => {
                if app.settings_field < max_idx { app.settings_field += 1; }
            }
            // Discord toggle
            KeyCode::Char(' ') | KeyCode::Enter if app.settings_field == DISCORD_TOGGLE_IDX => {
                app.discord_enabled = !app.discord_enabled;
            }
            // WhatsApp toggle
            KeyCode::Char(' ') | KeyCode::Enter if app.settings_field == TOGGLE_IDX => {
                app.wpp_enabled = !app.wpp_enabled;
            }
            // Open add custom category screen
            KeyCode::Enter if app.settings_field == CUSTOM_CAT_ADD_IDX => {
                app.custom_cat_step = 0;
                app.custom_cat_name.clear();
                app.custom_cat_icon.clear();
                app.screen = Screen::AddCustomCategory;
            }
            // Delete a custom category
            KeyCode::Char('d') | KeyCode::Char('D')
                if app.settings_field > CUSTOM_CAT_ADD_IDX =>
            {
                let cat_idx = app.settings_field - CUSTOM_CAT_ADD_IDX - 1;
                app.delete_custom_category(cat_idx);
                // Clamp settings_field
                let new_max = CUSTOM_CAT_ADD_IDX + app.custom_categories.len();
                if app.settings_field > new_max {
                    app.settings_field = new_max;
                }
            }
            // Edit a text field
            KeyCode::Enter if app.settings_field < SETTINGS_COUNT => {
                app.settings_editing = true;
            }
            KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
                app.save_settings()?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn handle_add_custom_category(app: &mut App, key: KeyCode) -> Result<()> {
    match key {
        KeyCode::Esc => {
            if app.custom_cat_step == 0 {
                app.screen = Screen::Settings;
            } else {
                app.custom_cat_step = 0;
            }
        }
        KeyCode::Enter => {
            if app.custom_cat_step == 0 {
                if !app.custom_cat_name.trim().is_empty() {
                    app.custom_cat_step = 1;
                }
            } else {
                app.add_custom_category();
                app.screen = Screen::Settings;
            }
        }
        KeyCode::Backspace => {
            if app.custom_cat_step == 0 {
                app.custom_cat_name.pop();
            } else {
                app.custom_cat_icon.pop();
            }
        }
        KeyCode::Char(c) => {
            if app.custom_cat_step == 0 {
                app.custom_cat_name.push(c);
            } else {
                app.custom_cat_icon.push(c);
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_wpp_setup(app: &mut App, key: KeyCode) -> Result<()> {
    let phase = app.wpp_state.lock().unwrap().phase.clone();

    match phase {
        WppPhase::AskPhone => match key {
            KeyCode::Esc => cancel_wpp_setup(app),
            KeyCode::Enter => save_wpp_phone(app)?,
            KeyCode::Backspace => {
                app.wpp_state.lock().unwrap().phone_input.pop();
            }
            KeyCode::Char(c) => {
                app.wpp_state.lock().unwrap().phone_input.push(c);
            }
            _ => {}
        },
        WppPhase::Done | WppPhase::Error(_) => {
            if key == KeyCode::Esc || key == KeyCode::Enter {
                app.screen = Screen::Dashboard;
            }
        }
        _ => {
            if key == KeyCode::Esc {
                cancel_wpp_setup(app);
            }
        }
    }
    Ok(())
}

fn cancel_wpp_setup(app: &mut App) {
    app.wpp_cancel.store(true, Ordering::Relaxed);
    app.screen = Screen::Dashboard;
}

fn save_wpp_phone(app: &mut App) -> Result<()> {
    let phone = app.wpp_state.lock().unwrap().phone_input.trim().to_string();
    if phone.is_empty() { return Ok(()); }

    let mut config = crate::config::load_config()?;
    config.whatsapp.tech_lead_phone = phone.clone();
    config.whatsapp.enabled = true;
    crate::config::save_config(&config)?;

    app.settings_inputs[crate::app::S_WPP_PHONE] = phone;
    app.wpp_enabled = true;

    app.wpp_state.lock().unwrap().phase = WppPhase::Done;
    app.wpp_state.lock().unwrap().log = "Número salvo! Tudo pronto.".to_string();
    Ok(())
}
