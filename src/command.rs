use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};

pub mod autocompleter;
pub mod history;

#[derive(Default, Clone)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    pub description: String,
}

impl Command {
    pub fn new(name: &str, args: Vec<String>, description: &str) -> Self {
        Self {
            name: name.to_string(),
            args: args,
            description: description.to_string(),
        }
    }
}
