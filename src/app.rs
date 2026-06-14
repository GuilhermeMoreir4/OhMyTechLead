use chrono::{Local, NaiveDate};
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

use crate::storage::{Category, Task, load_tasks, save_tasks};

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Dashboard,
    AddTask,
    PreviewReport,
    ConfirmSend,
    Settings,
    WhatsappSetup,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AddTaskStep {
    SelectCategory,
    EnterDescription,
}

// settings_inputs indices
pub const S_DISCORD_TOKEN: usize = 0;
pub const S_DISCORD_USER:  usize = 1;
pub const S_WPP_URL:       usize = 2;
pub const S_WPP_KEY:       usize = 3;
pub const S_WPP_INSTANCE:  usize = 4;
pub const S_WPP_PHONE:     usize = 5;
pub const S_SEND_TIME:     usize = 6;
pub const SETTINGS_COUNT:  usize = 7;
pub const DISCORD_TOGGLE_IDX: usize = SETTINGS_COUNT;     // 7
pub const TOGGLE_IDX:         usize = SETTINGS_COUNT + 1; // 8 (WhatsApp)

// ── WhatsApp setup state ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum WppPhase {
    Idle,
    CheckingDocker,
    StartingContainer,
    WaitingApi,
    CheckingInstance,
    CreatingInstance,
    ShowingQr,
    Connected,
    AskPhone,
    Done,
    Error(String),
}

pub struct WppState {
    pub phase: WppPhase,
    pub qr_rendered: Option<String>, // unicode-art QR code ready to display
    pub phone_input: String,
    pub log: String, // one-line status shown to the user
}

impl Default for WppState {
    fn default() -> Self {
        Self {
            phase: WppPhase::Idle,
            qr_rendered: None,
            phone_input: String::new(),
            log: String::new(),
        }
    }
}

// ── App ──────────────────────────────────────────────────────────────────────

pub struct App {
    pub screen: Screen,
    pub date: NaiveDate,
    pub tasks: Vec<Task>,

    pub selected_category: usize,
    pub selected_task: usize,

    pub add_step: AddTaskStep,
    pub add_category: usize,
    pub add_input: String,

    pub settings_field: usize,
    pub settings_inputs: Vec<String>,
    pub settings_editing: bool,
    pub discord_enabled: bool,
    pub wpp_enabled: bool,

    pub wpp_state: Arc<Mutex<WppState>>,
    pub wpp_cancel: Arc<AtomicBool>,

    pub status_message: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let date = Local::now().date_naive();
        let tasks = load_tasks(date)?;
        let config = crate::config::load_config()?;

        let mut inputs = vec![String::new(); SETTINGS_COUNT];
        inputs[S_DISCORD_TOKEN] = config.discord.bot_token.clone();
        inputs[S_DISCORD_USER]  = config.discord.tech_lead_user_id.clone();
        inputs[S_WPP_URL]       = config.whatsapp.evolution_url.clone();
        inputs[S_WPP_KEY]       = config.whatsapp.api_key.clone();
        inputs[S_WPP_INSTANCE]  = config.whatsapp.instance.clone();
        inputs[S_WPP_PHONE]     = config.whatsapp.tech_lead_phone.clone();
        inputs[S_SEND_TIME]     = config.schedule.send_time.clone();

        Ok(Self {
            screen: Screen::Dashboard,
            date,
            tasks,
            selected_category: 0,
            selected_task: 0,
            add_step: AddTaskStep::SelectCategory,
            add_category: 0,
            add_input: String::new(),
            settings_field: 0,
            settings_inputs: inputs,
            settings_editing: false,
            discord_enabled: config.discord.enabled,
            wpp_enabled: config.whatsapp.enabled,
            wpp_state: Arc::new(Mutex::new(WppState::default())),
            wpp_cancel: Arc::new(AtomicBool::new(false)),
            status_message: None,
            should_quit: false,
        })
    }

    pub fn tasks_for_category(&self, cat: &Category) -> Vec<&Task> {
        self.tasks.iter().filter(|t| &t.category == cat).collect()
    }

    pub fn selected_category_enum(&self) -> Category {
        Category::all()[self.selected_category].clone()
    }

    pub fn add_task(&mut self) {
        let cat = Category::all()[self.add_category].clone();
        let desc = self.add_input.trim().to_string();
        if !desc.is_empty() {
            self.tasks.push(Task::new(cat, desc));
            let _ = save_tasks(self.date, &self.tasks);
        }
        self.add_input.clear();
        self.add_step = AddTaskStep::SelectCategory;
        self.screen = Screen::Dashboard;
    }

    pub fn delete_selected_task(&mut self) {
        let cat = self.selected_category_enum();
        let cat_tasks: Vec<usize> = self
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.category == cat)
            .map(|(i, _)| i)
            .collect();
        if let Some(&idx) = cat_tasks.get(self.selected_task) {
            self.tasks.remove(idx);
            let _ = save_tasks(self.date, &self.tasks);
            if self.selected_task > 0 && self.selected_task >= cat_tasks.len().saturating_sub(1) {
                self.selected_task -= 1;
            }
        }
    }

    pub fn clamp_selected_task(&mut self) {
        let cat = self.selected_category_enum();
        let count = self.tasks_for_category(&cat).len();
        if count == 0 {
            self.selected_task = 0;
        } else if self.selected_task >= count {
            self.selected_task = count - 1;
        }
    }

    pub fn save_settings(&mut self) -> anyhow::Result<()> {
        let mut config = crate::config::load_config()?;
        config.discord.enabled            = self.discord_enabled;
        config.discord.bot_token          = self.settings_inputs[S_DISCORD_TOKEN].clone();
        config.discord.tech_lead_user_id  = self.settings_inputs[S_DISCORD_USER].clone();
        config.whatsapp.evolution_url     = self.settings_inputs[S_WPP_URL].clone();
        config.whatsapp.api_key           = self.settings_inputs[S_WPP_KEY].clone();
        config.whatsapp.instance          = self.settings_inputs[S_WPP_INSTANCE].clone();
        config.whatsapp.tech_lead_phone   = self.settings_inputs[S_WPP_PHONE].clone();
        config.whatsapp.enabled           = self.wpp_enabled;
        config.schedule.send_time         = self.settings_inputs[S_SEND_TIME].clone();
        crate::config::save_config(&config)?;
        self.set_status("Configurações salvas!");
        Ok(())
    }

    pub fn set_status(&mut self, msg: &str) {
        self.status_message = Some(msg.to_string());
    }

    /// Resets and returns (state, cancel) handles for a fresh WPP setup run.
    pub fn start_wpp_setup(&mut self) -> (Arc<Mutex<WppState>>, Arc<AtomicBool>) {
        use std::sync::atomic::Ordering;
        self.wpp_cancel.store(false, Ordering::Relaxed);
        *self.wpp_state.lock().unwrap() = WppState::default();
        (Arc::clone(&self.wpp_state), Arc::clone(&self.wpp_cancel))
    }
}
