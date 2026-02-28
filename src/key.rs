use std::io::{self, Read};

type KeyBuffer = [u8; 4];

pub(crate) enum Key {
    Char(char),
    Ctrl(char),
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Enter,
    Backspace,
    Escape,
    Unknown(KeyBuffer),
}

impl Key {
    /// Waits next key from user input
    pub(crate) fn read() -> io::Result<Self> {
        let mut buf = [0; 4];
        io::stdin().read(&mut buf)?;

        let key = match buf[0] {
            0x1b => match &buf[1..3] {
                b"\x00\x00" => Key::Escape,
                b"[A" => Key::ArrowUp,
                b"[B" => Key::ArrowDown,
                b"[C" => Key::ArrowRight,
                b"[D" => Key::ArrowLeft,
                _ => Key::Unknown(buf),
            },
            0x0d => Key::Enter,
            0x7f => Key::Backspace,
            b if b < 0x20 => Key::Ctrl((b + b'a' - 1) as char),
            b => Key::Char(b as char),
        };

        Ok(key)
    }
}
