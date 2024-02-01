use log::{info, log};

#[derive(Debug, Default)]
pub struct Player {
    id: u16,
    time: String,
    ping: u16,
    loss: u16,
    state: String,
    rate: u32,
    adr: String,
    name: String,
}

#[derive(Debug, Default)]
pub struct Status {
    servername: String,
    map: String,
    players: Vec<Player>,
}

#[derive(Debug)]
enum ParsingMode {
    None,
    Status,
    Map,
    Players,
}

impl Status {
    pub fn update(&mut self, st: String) {
        let mut lines = st.split("\n").collect::<Vec<&str>>();
        lines.reverse();
        info!("Updating status");
        let mut mode = ParsingMode::None;
        let mut spawn = "[0".to_string();
        while let Some(line) = lines.pop() {
            info!("line: {}, mode: {:?}", line, mode);
            match mode {
                ParsingMode::None => {
                    if line.starts_with("-") {
                        mode = ParsingMode::Status;
                    }
                }
                ParsingMode::Status => match line {
                    line if line.starts_with("hostname") => {
                        self.servername =
                            line.split(":").collect::<Vec<&str>>()[1].trim().to_string();
                    }
                    line if line.starts_with("spawn") => {
                        let mut newspawn = "[".to_string();
                        newspawn.push_str(line.split(":").collect::<Vec<&str>>()[1].trim());

                        spawn = newspawn;
                    }
                    line if line.starts_with("-") => {
                        mode = ParsingMode::Map;
                    }
                    _ => {}
                },
                ParsingMode::Map => match line {
                    line if line.contains(&spawn) => {
                        self.map = line.split(":").collect::<Vec<&str>>()[3]
                            .trim()
                            .to_string()
                            .split(" ")
                            .collect::<Vec<&str>>()[0]
                            .to_string();
                    }
                    line if line.starts_with("-") => {
                        mode = ParsingMode::Players;
                        let _ = lines.pop();
                    }
                    _ => {}
                },
                ParsingMode::Players => {
                    if line.starts_with("#end") {
                        mode = ParsingMode::None;
                    } else {
                        let mut player = Player::default();
                        let mut parts: Vec<String> =
                            line.split_whitespace().map(|e| e.to_string()).collect();
                        player.id = parts.remove(0).parse::<u16>().unwrap();
                        player.time = parts.remove(0);
                        player.ping = parts.remove(0).parse::<u16>().unwrap();
                        player.loss = parts.remove(0).parse::<u16>().unwrap();
                        player.state = parts.remove(0);
                        parts.reverse();
                        player.name = parts.remove(0);
                        player.adr = "".to_string();
                        player.rate = 0;
                        if parts.len() == 2 {
                            player.adr = parts.remove(0);
                            player.rate = parts.remove(0).parse::<u32>().unwrap();
                        } else if parts.len() == 1 {
                            player.adr = parts.remove(0).chars().skip(1).collect();
                        }

                        self.players.push(player);
                    }
                }
            }
        }
    }
}
