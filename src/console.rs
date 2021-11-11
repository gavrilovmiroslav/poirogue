use bracket_lib::prelude::{BLACK, BTerm, GREY, RGB, WHITE};
use bracket_lib::prelude::VirtualKeyCode;
use crate::world::{GameCommand, get_world, Tick};

pub struct Console {
    is_visible: bool,
    activation_key: VirtualKeyCode,
    buffer: String,
    history: Vec<String>,
}

impl Console {
    pub fn new() -> Console {
        Console {
            is_visible: false,
            activation_key: VirtualKeyCode::Grave,
            buffer: String::new(),
            history: Vec::new(),
        }
    }
}

pub fn virtual_key_to_char(v: VirtualKeyCode) -> Option<char> {
    match v {
        VirtualKeyCode::Key1 => Some('1'),
        VirtualKeyCode::Key2 => Some('2'),
        VirtualKeyCode::Key3 => Some('3'),
        VirtualKeyCode::Key4 => Some('4'),
        VirtualKeyCode::Key5 => Some('5'),
        VirtualKeyCode::Key6 => Some('6'),
        VirtualKeyCode::Key7 => Some('7'),
        VirtualKeyCode::Key8 => Some('8'),
        VirtualKeyCode::Key9 => Some('9'),
        VirtualKeyCode::Key0 => Some('0'),
        VirtualKeyCode::A => Some('A'),
        VirtualKeyCode::B => Some('B'),
        VirtualKeyCode::C => Some('C'),
        VirtualKeyCode::D => Some('D'),
        VirtualKeyCode::E => Some('E'),
        VirtualKeyCode::F => Some('F'),
        VirtualKeyCode::G => Some('G'),
        VirtualKeyCode::H => Some('H'),
        VirtualKeyCode::I => Some('I'),
        VirtualKeyCode::J => Some('J'),
        VirtualKeyCode::K => Some('K'),
        VirtualKeyCode::L => Some('L'),
        VirtualKeyCode::M => Some('M'),
        VirtualKeyCode::N => Some('N'),
        VirtualKeyCode::O => Some('O'),
        VirtualKeyCode::P => Some('P'),
        VirtualKeyCode::Q => Some('Q'),
        VirtualKeyCode::R => Some('R'),
        VirtualKeyCode::S => Some('S'),
        VirtualKeyCode::T => Some('T'),
        VirtualKeyCode::U => Some('U'),
        VirtualKeyCode::V => Some('V'),
        VirtualKeyCode::W => Some('W'),
        VirtualKeyCode::X => Some('X'),
        VirtualKeyCode::Y => Some('Y'),
        VirtualKeyCode::Z => Some('Z'),
        VirtualKeyCode::Space => Some(' '),
        VirtualKeyCode::Colon => Some(':'),
        VirtualKeyCode::Semicolon => Some(';'),
        VirtualKeyCode::Comma => Some(','),
        VirtualKeyCode::Plus => Some('+'),
        VirtualKeyCode::Asterisk => Some('*'),
        VirtualKeyCode::Period => Some('.'),
        VirtualKeyCode::Minus => Some('-'),
        VirtualKeyCode::Underline => Some('_'),
        VirtualKeyCode::Slash => Some('/'),
        VirtualKeyCode::LBracket => Some('['),
        VirtualKeyCode::RBracket => Some(']'),
        VirtualKeyCode::Backslash => Some('\\'),
        _ => None,
    }
}

pub fn interpret_game_command(s: &str) -> Option<GameCommand> {
    match s {
        "GEN" => Some(GameCommand::Gen),
        "EXIT" => Some(GameCommand::Exit),
        _ => None,
    }
}

impl Tick for Console {
    fn tick(&mut self, ctx: &mut BTerm) {
        if self.is_visible {
            ctx.draw_box(0, 0, 79, 15, RGB::named(GREY), RGB::named(GREY));
            ctx.draw_box(0, 0, 79, 3, RGB::named(GREY), RGB::named(GREY));
            ctx.print(3, 1, "> ");
            ctx.print(5, 1, &self.buffer);

            let mut i  = 0;
            for message in self.history.iter().rev().take(10) {
                ctx.print(3,5 + i, message);
                i += 1;
            }
        }

        match ctx.key {
            Some(x) if self.activation_key == x => {
                self.is_visible = !self.is_visible;
            }

            Some(VirtualKeyCode::Back) if self.is_visible && self.buffer.len() > 0 => {
                self.buffer.pop();
            },

            Some(VirtualKeyCode::Return) if self.is_visible && self.buffer.len() > 0 => {
                let comm = String::from(self.buffer.as_str());
                if let Some(command) = interpret_game_command(comm.as_str()) {
                    if let world = get_world() {
                        world.sender.send(command).unwrap();
                    }
                    self.history.push(format!("{:<20}        -- OK!", comm));
                } else {
                    self.history.push(format!("{:<20}        -- NOT RECOGNIZED", comm));
                }
                self.buffer.clear();
            },

            Some(v) if self.is_visible => {
                if let Some(k) = virtual_key_to_char(v) {
                    self.buffer.push(k);
                }
            },

            _ => {}
        }
    }
}