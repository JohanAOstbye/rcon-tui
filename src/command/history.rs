#[derive(Default)]
pub struct History {
    history: Vec<String>,
    index: usize,
}

impl History {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            index: 0,
        }
    }

    pub fn push(&mut self, command: String) {
        self.history.push(command);
        self.index = self.history.len();
    }

    pub fn get(&mut self, index: usize) -> Option<String> {
        if index < self.history.len() {
            Some(self.history[index].clone())
        } else {
            None
        }
    }

    pub fn backwards(&mut self) -> Option<String> {
        if self.index < self.history.len() {
            self.index += 1;
            self.get(self.index)
        } else {
            None
        }
    }

    pub fn forwards(&mut self) -> Option<String> {
        if self.index > 0 {
            self.index -= 1;
            self.get(self.index)
        } else {
            None
        }
    }
}
