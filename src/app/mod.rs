use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::prelude::*;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::backend::Backend;
use crate::telemetry::TelemetryData;
use crate::ui::{Tab, draw_ui};

pub struct App {
    should_quit: bool,
    current_tab: Tab,
    backend: Backend,
    telemetry_data: Vec<TelemetryData>,
    selected_device: usize,
    show_help: bool,
    show_sidebar: bool,
    dark_mode: bool,
}

impl App {
    pub async fn new(compact_mode: bool) -> Result<Self> {
        let backend = Backend::new().await?;
        let telemetry_data = backend.get_initial_data().await?;

        Ok(Self {
            should_quit: false,
            current_tab: Tab::DeviceInfo,
            backend,
            telemetry_data,
            selected_device: 0,
            show_help: false,
            show_sidebar: !compact_mode,
            dark_mode: true,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = self.setup_terminal()?;

        let (tx, mut rx) = mpsc::unbounded_channel();

        let update_tx = tx.clone();
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(100));
            loop {
                ticker.tick().await;
                let _ = update_tx.send(AppEvent::UpdateTelemetry);
            }
        });

        loop {
            terminal.draw(|f| draw_ui(f, self))?;

            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if self.handle_key_event(key)? {
                        break;
                    }
                }
            }

            while let Ok(event) = rx.try_recv() {
                match event {
                    AppEvent::UpdateTelemetry => {
                        self.update_telemetry().await?;
                    }
                }
            }
        }

        self.restore_terminal(&mut terminal)?;
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        if std::mem::take(&mut self.show_help) {
            return Ok(false);
        }

        Ok(match key.code {
            KeyCode::Char('q') => true,
            KeyCode::Char('h') => {
                self.show_help = true;
                false
            }
            KeyCode::Char('d') => {
                self.dark_mode = !self.dark_mode;
                false
            }
            KeyCode::Char('c') => {
                self.show_sidebar = !self.show_sidebar;
                false
            }
            KeyCode::Char('1') => {
                self.current_tab = Tab::DeviceInfo;
                false
            }
            KeyCode::Char('2') => {
                self.current_tab = Tab::Telemetry;
                false
            }
            KeyCode::Char('3') => {
                self.current_tab = Tab::Firmware;
                false
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.previous_device();
                false
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.next_device();
                false
            }
            _ => false,
        })
    }

    async fn update_telemetry(&mut self) -> Result<()> {
        self.telemetry_data = self.backend.get_telemetry_update().await?;
        Ok(())
    }

    fn next_device(&mut self) {
        if !self.telemetry_data.is_empty() {
            self.selected_device = (self.selected_device + 1) % self.telemetry_data.len();
        }
    }

    fn previous_device(&mut self) {
        if !self.telemetry_data.is_empty() {
            self.selected_device = self
                .selected_device
                .checked_sub(1)
                .unwrap_or(self.telemetry_data.len().saturating_sub(1));
        }
    }

    fn setup_terminal(&self) -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        crossterm::execute!(
            stdout,
            crossterm::terminal::EnterAlternateScreen,
            crossterm::event::EnableMouseCapture
        )?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(terminal)
    }

    fn restore_terminal(
        &self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    }

    pub fn get_current_tab(&self) -> Tab {
        self.current_tab
    }

    pub fn get_telemetry_data(&self) -> &[TelemetryData] {
        &self.telemetry_data
    }

    pub fn get_selected_device(&self) -> usize {
        self.selected_device
    }

    pub fn is_dark_mode(&self) -> bool {
        self.dark_mode
    }

    pub fn should_show_sidebar(&self) -> bool {
        self.show_sidebar
    }

    pub fn should_show_help(&self) -> bool {
        self.show_help
    }
}

enum AppEvent {
    UpdateTelemetry,
}
