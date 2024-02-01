use ratatui::{
    style::Color,
    widgets::{Block, Borders, Paragraph},
};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};

pub mod autocompleter;
pub mod history;
pub mod status;
use crate::popup::Popup;

#[derive(Default, Clone)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub flags: Vec<String>,
}

impl Command {
    pub fn new(name: &str, description: &str, flags: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            flags,
        }
    }

    pub fn widget(&mut self) -> Popup<'_> {
        Popup::default()
            .title(self.name.as_str())
            .content(self.description.as_str())
    }
}
