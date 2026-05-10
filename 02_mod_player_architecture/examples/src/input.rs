use crossterm::event::{Event, KeyCode, KeyEvent, read};

pub enum InputCommand {
    Play,
    Stop,
    VolumeUp,
    VolumeDown,
}

pub struct InputProcessor {
    command: InputCommand,
}

impl InputProcessor {
    pub fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            command: InputCommand::Stop,
        })
    }

    pub fn key_board_input() -> Result<InputCommand, anyhow::Error> {
        loop {
            if let Event::Key(KeyEvent { code, .. }) = read()? {
                match code {
                    KeyCode::Char('p') => return Ok(InputCommand::Play),
                    KeyCode::Char('s') => return Ok(InputCommand::Stop),
                    KeyCode::Down => return Ok(InputCommand::VolumeDown),
                    KeyCode::Up => return Ok(InputCommand::VolumeUp),
                    _ => continue,
                }
            }
        }
    }
}
