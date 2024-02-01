use crossterm::event::{Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};

use tui_input::{backend::crossterm::EventHandler, Input, StateChanged};

use crate::command::{autocompleter::AutoCompleter, history::History, Command};

pub struct Inputwrapper {
    input: Input,
    history: History,
    auto_completer: AutoCompleter,
    pub suggestion: Option<String>,
}

impl Inputwrapper {
    pub fn new() -> Self {
        Self {
            input: Input::default(),
            history: History::new(),
            auto_completer: AutoCompleter::default(),
            suggestion: None,
        }
    }

    pub fn init(&mut self) {
        self.auto_completer.load_commands(".config/commands.txt");
        self.auto_completer.load_commands(".config/convars.txt");
    }

    pub fn value(&self) -> &str {
        self.input.value()
    }

    pub fn reset(&mut self) {
        self.input.reset();
    }

    pub fn cursor(&self) -> usize {
        self.input.cursor()
    }

    pub fn visual_scroll(&self, width: usize) -> usize {
        self.input.visual_scroll(width)
    }

    pub fn handle_event(&mut self, event: &Event) -> Option<StateChanged> {
        let state_changed = match event {
            Event::Key(key) => match key.code {
                KeyCode::Up => {
                    self.forwards();
                    Some(StateChanged {
                        value: false,
                        cursor: false,
                    })
                }
                KeyCode::Down => {
                    self.backwards();
                    Some(StateChanged {
                        value: false,
                        cursor: false,
                    })
                }
                KeyCode::Tab => {
                    self.accept_suggestion();
                    Some(StateChanged {
                        value: true,
                        cursor: true,
                    })
                }
                _ => self.input.handle_event(event),
            },
            _ => self.input.handle_event(event),
        };
        match state_changed {
            Some(state_changed) => {
                if state_changed.value {
                    self.update_suggestion();
                }
                Some(state_changed)
            }
            _ => None,
        }
    }

    pub fn forwards(&mut self) {
        if let Some(command) = self.history.forwards() {
            self.update_suggestion();
            self.input = Input::new(command);
        }
    }

    pub fn backwards(&mut self) {
        match self.history.backwards() {
            Some(command) => {
                self.update_suggestion();
                self.input = Input::new(command);
            }
            _ => {
                self.input.reset();
                self.update_suggestion();
            }
        }
    }

    pub fn push_history(&mut self, command: String) {
        self.history.push(command);
    }

    pub fn update_suggestion(&mut self) {
        let command_parts = self.input.value().split(' ').collect::<Vec<&str>>();
        let value = command_parts[0];
        match value {
            "" => {
                self.suggestion = None;
            }
            _ if value == self.suggestion.clone().unwrap_or_default() => {}
            _ => {
                self.suggestion = self
                    .auto_completer
                    .get_suggestion(value, self.suggestion.as_deref());
                log::info!("Suggestion: {:?}", self.suggestion);
            }
        }
    }

    pub fn accept_suggestion(&mut self) {
        if let Some(suggestion) = &self.suggestion {
            self.input = Input::new(suggestion.clone());
        }
    }

    pub fn get_current_command(&self) -> Option<Command> {
        let command_parts = self.input.value().split(' ').collect::<Vec<&str>>();
        let value = command_parts[0];
        self.auto_completer.get_command(value)
    }
}

impl Default for Inputwrapper {
    fn default() -> Self {
        let mut inputwrapper = Self::new();
        inputwrapper.init();
        inputwrapper
    }
}
