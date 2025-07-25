#[cfg(unix)]
pub fn terminal_size() -> Option<(u16, u16)> {
    use libc::{TIOCGWINSZ, ioctl};
    use std::io;
    use std::mem::MaybeUninit;
    use std::os::fd::AsRawFd;

    #[repr(C)]
    struct Winsize {
        ws_row: u16,
        ws_col: u16,
        ws_xpixel: u16,
        ws_ypixel: u16,
    }

    let stdout = io::stdout();
    let fd_stdout = stdout.as_raw_fd();

    let mut size: MaybeUninit<Winsize> = MaybeUninit::uninit();

    unsafe {
        if ioctl(fd_stdout, TIOCGWINSZ, size.as_mut_ptr()) != -1 {
            let size = size.assume_init();
            Some((size.ws_col, size.ws_row))
        } else {
            None
        }
    }
}

#[cfg(windows)]
pub fn terminal_size() -> Option<(u16, u16)> {
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::wincon::{CONSOLE_SCREEN_BUFFER_INFO, GetConsoleScreenBufferInfo};

    unsafe {
        let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();

        if GetConsoleScreenBufferInfo(GetStdHandle(-11i32 as _), &mut csbi) != 0 {
            let width = csbi.srWindow.Right - csbi.srWindow.Left + 1;
            let height = csbi.srWindow.Bottom - csbi.srWindow.Top + 1;
            Some((width as u16, height as u16))
        } else {
            None
        }
    }
}

#[cfg(not(any(unix, windows)))]
pub fn terminal_size() -> Option<(u16, u16)> {
    None // Unsupported platforms
}
