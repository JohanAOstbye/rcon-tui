use std::path::Path;

use futures::future::ok;
use ratatui::{prelude::*, widgets::*};
use rcon::{AsyncStdStream, Connection, Error};
use tokio::{
    sync::mpsc::UnboundedSender,
    time::{timeout, Duration},
};

use crate::{action::Action, command::status::Status};
use log::{error, info};

pub struct Client {
    connection: Option<Connection<AsyncStdStream>>,
    address: String,
    password: String,
    action_tx: Option<UnboundedSender<Action>>,
    status_rate: usize,
    ticks: usize,
    status: Status,
}

impl Client {
    pub async fn new(address: &str, password: &str) -> Self {
        Self {
            connection: None,
            address: address.to_string(),
            password: password.to_string(),
            action_tx: None,
            status_rate: 20,
            ticks: 0,
            status: Status::default(),
        }
    }

    pub fn register_action_handler(
        &mut self,
        tx: UnboundedSender<Action>,
    ) -> color_eyre::eyre::Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    pub fn set_address(&mut self, address: &str) {
        self.address = address.to_string();
    }

    pub fn set_password(&mut self, password: &str) {
        self.password = password.to_string();
    }

    pub async fn connect(&mut self) -> Result<(), Error> {
        self.connection = Some(
            <Connection<AsyncStdStream>>::builder()
                .connect(&self.address, &self.password)
                .await?,
        );
        match self.is_connected() {
            true => Ok(()),
            false => Err(Error::Auth),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    async fn send_command(&mut self, command: &str) -> Result<String, Error> {
        match self.connection.as_mut() {
            Some(connection) => {
                log::info!("Sending command: {}", command);
                let response = connection.cmd(command).await?;
                log::info!("Response: {}", response);
                self.send_action(Action::Insert(response.clone()));
                Ok(response)
            }
            None => {
                if self.address.is_empty() {
                    self.error("No address specified".to_owned()).await?;
                    return Err(Error::Auth);
                }
                if self.password.is_empty() {
                    self.error("No password specified".to_owned()).await?;
                    Err(Error::Auth)
                } else {
                    self.error("Not connected".to_owned()).await?;
                    Err(Error::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Not connected",
                    )))
                }
            }
        }
    }

    async fn run_file(&mut self, file: &str) -> Result<Vec<String>, Error> {
        let path = format!("cfg/{}.cfg", file);
        let contents = tokio::fs::read_to_string(path).await?;
        let mut responses = Vec::new();
        for command in contents.split('\n') {
            let response = self.send_command(command).await?;
            log::info!("command {}:\n{}", command, response);
            responses.push(response);
        }
        Ok(responses)
    }

    pub async fn run_command(&mut self, command: &str) -> Result<(), Error> {
        match command.split(' ').collect::<Vec<&str>>().first() {
            Some(&"connect") => {
                let args = command.split(' ').collect::<Vec<&str>>();
                if args.len() < 2 {
                    self.error("Not enough arguments".to_owned()).await?;
                    return Ok(());
                }
                let address = args[1];

                let password = match args.len() {
                    3 => args[2],
                    2 => "",
                    _ => {
                        self.error("Too many arguments".to_owned()).await?;
                        return Ok(());
                    }
                };
                log::info!("Connecting to {}", address);
                self.set_address(address);
                self.set_password(password);
                match self.connect().await {
                    Ok(_) => {
                        self.send_action(Action::Connected(true));
                    }
                    Err(e) => {
                        self.error(format!("Failed to connect: {:?}", e)).await?;
                    }
                };
            }
            Some(&"disconnect") => {
                log::info!("Disconnecting");
                self.connection = None;
                self.send_action(Action::Connected(false));
            }
            Some(&"exec") => {
                let args = command.split(' ').collect::<Vec<&str>>();
                if args.len() < 2 {
                    self.error("Not enough arguments".to_owned()).await?;
                    return Ok(());
                }
                let file = args[1];
                log::info!("Executing file: {}.cfg", file);
                let responses = self.run_file(file).await;
            }
            _ => {
                log::info!("Running command: {}", command);
                let response = self.send_command(command).await?;
            }
        }
        Ok(())
    }

    pub async fn async_update(&mut self, action: Action) {
        match action {
            Action::Command(command) => {
                let _ = self.run_command(&command).await;
            }
            Action::Tick => {
                self.ticks += 1;
                info!("Ticks: {}", self.ticks);
                if self.ticks % self.status_rate == 0 {
                    info!("Updating status");
                    if let Some(connection) = self.connection.as_mut() {
                        info!("Sending status command");
                        match timeout(Duration::from_secs(5), connection.cmd("status")).await {
                            Ok(status) => match status {
                                Ok(status) => {
                                    self.status.update(status);
                                }
                                Err(_) => {
                                    self.connection = None;
                                    self.send_action(Action::Connected(false));
                                }
                            },
                            Err(_) => {
                                self.connection = None;
                                self.send_action(Action::Connected(false));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub async fn error(&mut self, error: String) -> Result<(), Error> {
        error!("Error: {}", error);
        self.send_action(Action::Error(error.clone()));
        Ok(())
    }

    pub fn send_action(&mut self, action: Action) {
        if let Some(sender) = &self.action_tx {
            log::info!("Sending action: {:?}", action);
            if let Err(e) = sender.send(action) {
                error!("Failed to send action: {:?}", e);
            }
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self {
            connection: None,
            address: String::new(),
            password: String::new(),
            action_tx: None,
            status_rate: 20,
            ticks: 0,
            status: Status::default(),
        }
    }
}
