use std::{collections::HashMap, process::Command, time::Duration};

use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use log::error;
use ratatui::{
    prelude::*,
    widgets::{block::Title, *},
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::backend::crossterm::EventHandler;

use super::{Component, Frame};
use crate::{action::Action, config::key_event_to_string, inputwrapper::Inputwrapper};

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Insert,
    Normal,
    Processing,
    Help,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    #[default]
    Normal,
    Hover,
    Clicked,
}

#[derive(Default)]
pub struct Home {
    pub show_help: bool,
    pub app_ticker: usize,
    pub render_ticker: usize,
    pub mode: Mode,
    pub input: Inputwrapper,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: HashMap<KeyEvent, Action>,
    pub text: Vec<String>,
    pub connected: bool,
    pub error: Option<String>,
    pub last_events: Vec<KeyEvent>,
    pub main_rect: Rect,
    pub input_rect: Rect,
}

impl Home {
    pub fn new() -> Self {
        Self::default().connected(false)
    }

    pub fn connected(mut self, connected: bool) -> Self {
        self.connected = connected;
        self
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }

    pub fn tick(&mut self) {
        //log::info!("Tick");
        self.app_ticker = self.app_ticker.saturating_add(1);
        self.last_events.drain(..);
    }

    pub fn render_tick(&mut self) {
        //log::debug!("Render Tick");
        self.render_ticker = self.render_ticker.saturating_add(1);
    }

    pub fn insert(&mut self, s: String) {
        self.text.insert(0, s)
    }

    pub fn remove(&mut self) {
        self.text.pop();
    }

    pub fn main_widget(&mut self) -> Paragraph<'_> {
        let mut text: Vec<Line> = self
            .text
            .clone()
            .iter()
            .map(|l| Line::from(l.clone()))
            .collect();
        text.insert(0, "".into());
        text.insert(
            0,
            match self.error.clone() {
                Some(e) => Line::from(vec![Span::styled(
                    format!("Error: {}", e),
                    Style::default().fg(Color::Red),
                )]),
                None => Line::from(vec!["".into()]),
            },
        );
        text.insert(0, "".into());
        text.insert(0, "Type commands under and hit enter".dim().into());
        text.insert(0, "".into());
        text.insert(
            0,
            Line::from(vec![
                "Connect with the command: ".into(),
                Span::styled("connect <ip>", Style::default().fg(Color::Red)),
                ":".into(),
                Span::styled("<port>", Style::default().fg(Color::Red)),
                " (".into(),
                Span::styled("<password>", Style::default().fg(Color::Yellow)),
                ")".into(),
            ]),
        );
        text.insert(
            0,
            Line::from(vec![match self.connected {
                true => Span::styled("Connected", Style::default().fg(Color::Green)),
                false => Span::styled("Not Connected", Style::default().fg(Color::Red)),
            }]),
        );
        text.insert(0, "".into());
        Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(match self.mode {
                        Mode::Processing => Style::default().fg(Color::Yellow),
                        _ => Style::default(),
                    })
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false })
    }

    fn input_widget(&mut self) -> Paragraph<'_> {
        let width = self.main_rect.width.max(3) - 3; // keep 2 for borders and 1 for cursor
        let scroll = self.input.visual_scroll(width as usize);
        let text: Line = match self.input.value().is_empty() {
            true => Line::from(""),
            false => match &self.input.suggestion {
                Some(s) => {
                    let command_offset = self.input.value().len().min(s.len());
                    let suggestion: String =
                        (s.clone().drain(command_offset..).collect::<String>()).to_string();
                    Line::from(vec![
                        Span::raw(self.input.value()),
                        Span::from(suggestion).dim(),
                    ])
                }
                _ => Line::from(self.input.value()),
            },
        };
        Paragraph::new(text)
            .style(match self.mode {
                Mode::Insert => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
            .scroll((0, scroll as u16))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title_alignment(Alignment::Right)
                    .title_position(block::Position::Bottom)
                    .title(Line::from(vec![
                        Span::raw("Enter Input Mode "),
                        Span::styled("(Press ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            "/",
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Gray),
                        ),
                        Span::styled(" to start, ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            "Enter",
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Gray),
                        ),
                        Span::styled(" to send, ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            "ESC",
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Gray),
                        ),
                        Span::styled(" to finish)", Style::default().fg(Color::DarkGray)),
                    ])),
            )
    }

    fn help_widget(&mut self) -> (Block<'_>, Table<'_>) {
        let block = Block::default()
            .title(Line::from(vec![Span::styled(
                "Key Bindings",
                Style::default().add_modifier(Modifier::BOLD),
            )]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let rows = vec![
            Row::new(vec!["/", "Enter Input"]),
            Row::new(vec!["ESC", "Exit Input"]),
            Row::new(vec!["Enter", "Submit Input"]),
            Row::new(vec!["q", "Quit"]),
            Row::new(vec!["?", "Open Help"]),
        ];
        let table = Table::new(
            rows,
            [Constraint::Percentage(10), Constraint::Percentage(90)],
        )
        .header(
            Row::new(vec!["Key", "Action"])
                .bottom_margin(1)
                .style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .column_spacing(1);
        (block, table)
    }

    fn title_widget(&mut self) -> Block<'_> {
        Block::default()
            .title(
                Title::from(format!(
                    "{:?}",
                    &self
                        .last_events
                        .iter()
                        .map(key_event_to_string)
                        .collect::<Vec<_>>()
                ))
                .alignment(Alignment::Right),
            )
            .title_style(Style::default().add_modifier(Modifier::BOLD))
    }
}

impl Component for Home {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        self.last_events.push(key);
        let action = match self.mode {
            Mode::Normal | Mode::Processing | Mode::Help => return Ok(None),
            Mode::Insert => match key.code {
                KeyCode::Esc => Action::EnterNormal,
                KeyCode::Enter => {
                    self.error = None;
                    self.input.push_history(self.input.value().to_string());
                    if let Some(sender) = &self.action_tx {
                        log::info!(
                            "Sending action: {:?}",
                            Action::Command(self.input.value().to_string())
                        );
                        if let Err(e) = sender.send(Action::Command(self.input.value().to_string()))
                        {
                            error!("Failed to send action: {:?}", e);
                        }
                        self.input.reset();
                    }
                    Action::Update
                }
                _ => {
                    self.input.handle_event(&crossterm::event::Event::Key(key));
                    Action::Update
                }
            },
        };
        Ok(Some(action))
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => self.tick(),
            Action::Render => self.render_tick(),
            Action::ToggleShowHelp if self.mode != Mode::Insert => {
                self.show_help = !self.show_help;
                if self.show_help {
                    self.mode = Mode::Help;
                } else {
                    self.mode = Mode::Normal;
                }
            }
            Action::Insert(s) => self.insert(s),
            Action::EnterNormal => {
                self.mode = Mode::Normal;
            }
            Action::EnterInsert => {
                self.mode = Mode::Insert;
            }
            Action::EnterProcessing => {
                self.mode = Mode::Processing;
            }
            Action::ExitProcessing => {
                // TODO: Make this go to previous mode instead
                self.mode = Mode::Normal;
            }
            Action::Connected(connected) => {
                self.connected = connected;
            }
            Action::Error(e) => {
                self.error = Some(e);
            }
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
        let [input_rect, main_rect] = *Layout::default()
            .constraints([Constraint::Min(3), Constraint::Percentage(100)].as_ref())
            .split(rect)
        else {
            panic!("Unable to split rects into a refutable pattern");
        };

        f.render_widget(self.main_widget(), main_rect);
        self.main_rect = main_rect;

        f.render_widget(self.input_widget(), input_rect);
        self.input_rect = input_rect;

        if self.mode == Mode::Insert {
            f.set_cursor(
                (input_rect.x + 1 + self.input.cursor() as u16)
                    .min(input_rect.x + input_rect.width - 2),
                input_rect.y + 1,
            )
        }

        if self.show_help {
            let rect = rect.inner(&Margin {
                horizontal: 4,
                vertical: 2,
            });
            f.render_widget(Clear, rect);
            let (block, table) = self.help_widget();
            f.render_widget(block, rect);
            f.render_widget(
                table,
                rect.inner(&Margin {
                    vertical: 4,
                    horizontal: 2,
                }),
            );
        };

        f.render_widget(
            self.title_widget(),
            Rect {
                x: rect.x + 1,
                y: rect.height.saturating_sub(1),
                width: rect.width.saturating_sub(2),
                height: 1,
            },
        );

        // draw title
        f.render_widget(
            Block::new()
                .title("rcon-client")
                .title_alignment(Alignment::Center),
            f.size(),
        );

        // draw command descriptions
        if let Some(mut command) = self.input.get_current_command() {
            f.render_widget(
                command.widget(),
                Rect {
                    x: 0,
                    y: 3,
                    width: 40,
                    height: 10,
                },
            );
        }
        Ok(())
    }
}
