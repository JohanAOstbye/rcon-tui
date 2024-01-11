use crate::command::{CommandAutoCompleter, CommandHistory};
use crossterm::event::Event;
use tui_input::{backend::crossterm::EventHandler, Input, StateChanged};

#[derive(Default)]
pub struct Inputwrapper {
    input: Input,
    history: CommandHistory,
    suggester: CommandAutoCompleter,
    pub suggestion: Option<String>,
}

impl Inputwrapper {
    pub fn new() -> Self {
        Self {
            input: Input::default(),
            history: CommandHistory::new(),
            suggester: CommandAutoCompleter::default(),
            suggestion: None,
        }
    }

    pub fn value(&self) -> &str {
        return self.input.value();
    }

    pub fn reset(&mut self) {
        self.input.reset();
    }

    pub fn cursor(&self) -> usize {
        return self.input.cursor();
    }

    pub fn visual_scroll(&self, width: usize) -> usize {
        return self.input.visual_scroll(width);
    }

    pub fn handle_event(&mut self, event: &Event) -> Option<StateChanged> {
        let state_changed = self.input.handle_event(event);
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

    pub fn prev(&mut self) {
        match self.history.prev() {
            Some(command) => {
                self.update_suggestion();
                self.input = Input::new(command);
            }
            _ => {}
        }
    }

    pub fn next(&mut self) {
        match self.history.next() {
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
        let value = self.input.value();
        match value {
            "" => {
                self.suggestion = None;
            }
            _ => {
                self.suggestion = self.suggester.get_suggestion(value);
                log::info!("Suggestion: {:?}", self.suggestion);
            }
        }
    }
}
