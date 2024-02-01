use crate::command::Command;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path,
};

pub struct AutoCompleter {
    commands: Vec<Command>,
}

impl AutoCompleter {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn add_command(&mut self, command: Command) {
        self.commands.push(command);
    }

    pub fn load_commands(&mut self, path: &str) {
        let file = File::open(path).unwrap();
        let lines = BufReader::new(file).lines();
        for line in lines.into_iter().flatten() {
            let command: Vec<&str> = line.split('$').collect();
            let name = command[0];
            let description = command[1];
            if command.len() < 3 {
                self.add_command(Command::new(name, description, Vec::new()));
                continue;
            }
            let flags = command[2]
                .trim()
                .split(' ')
                .map(|e| e.to_string())
                .collect::<Vec<String>>();
            self.add_command(Command::new(name, description, flags));
        }
    }

    pub fn get_suggestions(&self, partial_command: &str, count: usize) -> Vec<String> {
        let mut completions = Vec::new();
        for command in self.commands.iter() {
            if completions.len() >= count {
                break;
            }
            if command.name.starts_with(partial_command) {
                completions.push(command.name.clone());
            }
        }
        log::info!("Found {} completions", completions.len());
        completions
    }

    pub fn get_suggestion(&self, partial: &str, current: Option<&str>) -> Option<String> {
        if let Some(current) = current {
            for command in self.commands.iter() {
                if command.name.starts_with(partial) {
                    return Some(command.name.clone());
                }
            }
        }
        for command in self.commands.iter() {
            if command.name.starts_with(partial) {
                return Some(command.name.clone());
            }
        }
        None
    }

    pub fn get_command(&self, name: &str) -> Option<Command> {
        for command in self.commands.iter() {
            if command.name == name {
                return Some(command.clone());
            }
        }
        None
    }
}

impl Default for AutoCompleter {
    fn default() -> Self {
        Self::new()
    }
}
