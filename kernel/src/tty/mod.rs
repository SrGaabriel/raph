use crate::println;

pub mod color;
mod font;
pub mod keyboard;
pub mod writer;

pub struct Tty {
    pub writer: writer::TextWriter,
    pub command_handler: Option<fn(&str)>,
}

impl Tty {
    pub fn new(writer: writer::TextWriter, command_handler: Option<fn(&str)>) -> Self {
        Self {
            writer,
            command_handler,
        }
    }

    pub fn handle_input(&mut self, input: u8) {
        let key_event = keyboard::KeyEvent::from_scancode(input);
        println!("Received key event: {:?}", key_event);
    }

    pub fn set_command_handler(&mut self, handler: Option<fn(&str)>) {
        self.command_handler = handler;
    }
}

