use libc::{
    CS8, ECHO, ICANON, ICRNL, IEXTEN, ISIG, IXON, OPOST, STDIN_FILENO, TCSAFLUSH, VMIN, VTIME,
    tcgetattr, tcsetattr, termios,
};

pub(crate) struct RawMode(termios);

impl RawMode {
    pub(crate) fn new() -> Self {
        let mut termios = unsafe { std::mem::zeroed::<termios>() };
        unsafe { tcgetattr(STDIN_FILENO, &mut termios) };
        // save current term state
        let src = termios;

        // raw mode opts
        termios.c_iflag &= !(IXON | ICRNL);
        termios.c_oflag &= !OPOST;
        termios.c_lflag &= !(ECHO | ICANON | ISIG | IEXTEN);
        termios.c_cflag |= CS8;
        termios.c_cc[VMIN] = 1;
        termios.c_cc[VTIME] = 0;

        unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &termios) };
        Self(src)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        // restore term state
        unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &self.0) };
    }
}
