use core::fmt::Write;

use crate::{tty::keyboard::{Key, KeyEvent}};

pub mod color;
mod font;
pub mod keyboard;
pub mod writer;

pub struct Tty {
    pub writer: writer::TextWriter,
    pub command_handler: Option<fn(&str)>,
    control_key_pressed: bool,
    shift_key_pressed: bool,
    alt_key_pressed: bool,
    caps_lock_active: bool,
}

impl Tty {
    pub fn new(writer: writer::TextWriter, command_handler: Option<fn(&str)>) -> Self {
        Self {
            writer,
            command_handler,
            control_key_pressed: false,
            shift_key_pressed: false,
            alt_key_pressed: false,
            caps_lock_active: false,
        }
    }

    pub fn handle_input(&mut self, input: u8) {
        let key_event = KeyEvent::from_scancode(input);
        match key_event {
            KeyEvent::Pressed(Key::LeftShift) => self.shift_key_pressed = true,
            KeyEvent::Released(Key::LeftShift) => self.shift_key_pressed = false,
            KeyEvent::Pressed(Key::RightShift) => self.shift_key_pressed = true,
            KeyEvent::Released(Key::RightShift) => self.shift_key_pressed = false,
            KeyEvent::Pressed(Key::LeftCtrl) => self.control_key_pressed = true,
            KeyEvent::Released(Key::LeftCtrl) => self.control_key_pressed = false,
            KeyEvent::Pressed(Key::RightCtrl) => self.control_key_pressed = true,
            KeyEvent::Released(Key::RightCtrl) => self.control_key_pressed = false,
            KeyEvent::Pressed(Key::LeftAlt) => self.alt_key_pressed = true,
            KeyEvent::Released(Key::LeftAlt) => self.alt_key_pressed = false,
            KeyEvent::Pressed(Key::RightAlt) => self.alt_key_pressed = true,
            KeyEvent::Released(Key::RightAlt) => self.alt_key_pressed = false,
            KeyEvent::Pressed(Key::CapsLock) => self.caps_lock_active = !self.caps_lock_active,
            KeyEvent::Pressed(u) => {
                let action = self.interpret_key(u);
                self.perform_action(action);
            },
            KeyEvent::Released(_) => {}
        }
    }

    pub fn perform_action(&mut self, action: KeyAction) {
        match action {
            KeyAction::Type(c) => {
                self.writer.write_char(c);
            },
            _ => {}
        }
    }
    
    pub fn set_command_handler(&mut self, handler: Option<fn(&str)>) {
        self.command_handler = handler;
    }
    
    fn interpret_key(&self, key: Key) -> KeyAction {
        match key {
            Key::Backspace => KeyAction::Backspace,
            Key::Enter => KeyAction::Enter,
            Key::Tab => KeyAction::Tab,
            Key::Up => KeyAction::ArrowUp,
            Key::Down => KeyAction::ArrowDown,
            Key::Left => KeyAction::ArrowLeft,
            Key::Right => KeyAction::ArrowRight,
            c => match c.to_ascii(self.is_uppercase()) {
                Some(c) => KeyAction::Type(c as char), // todo review
                None => KeyAction::Ignore,
            }
            _ => KeyAction::Ignore,
        }
    }
    
    fn is_uppercase(&self) -> bool {
        self.shift_key_pressed ^ self.caps_lock_active
    }
}

#[derive(Debug)]
pub enum KeyAction {
    Type(char),
    Backspace,
    Enter,
    Tab,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Ignore
}
