#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sgr {
    Reset,

    // foreground
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,

    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,

    // background
    BgBlack,
    BgRed,
    BgGreen,
    BgYellow,
    BgBlue,
    BgMagenta,
    BgCyan,
    BgWhite,

    BgBrightBlack,
    BgBrightRed,
    BgBrightGreen,
    BgBrightYellow,
    BgBrightBlue,
    BgBrightMagenta,
    BgBrightCyan,
    BgBrightWhite,

    // styles
    Bold,
    Dim,
    Italic,
    Underline,
    Blink,
    Reverse,
    Hidden,
    Strikethrough,
}

impl Sgr {
    // pub const fn code(self) -> u8 {
    //     match self {
    //         Self::Reset => 0,
    //
    //         Self::Black => 30,
    //         Self::Red => 31,
    //         Self::Green => 32,
    //         Self::Yellow => 33,
    //         Self::Blue => 34,
    //         Self::Magenta => 35,
    //         Self::Cyan => 36,
    //         Self::White => 37,
    //
    //         Self::BrightBlack => 90,
    //         Self::BrightRed => 91,
    //         Self::BrightGreen => 92,
    //         Self::BrightYellow => 93,
    //         Self::BrightBlue => 94,
    //         Self::BrightMagenta => 95,
    //         Self::BrightCyan => 96,
    //         Self::BrightWhite => 97,
    //
    //         Self::BgBlack => 40,
    //         Self::BgRed => 41,
    //         Self::BgGreen => 42,
    //         Self::BgYellow => 43,
    //         Self::BgBlue => 44,
    //         Self::BgMagenta => 45,
    //         Self::BgCyan => 46,
    //         Self::BgWhite => 47,
    //
    //         Self::BgBrightBlack => 100,
    //         Self::BgBrightRed => 101,
    //         Self::BgBrightGreen => 102,
    //         Self::BgBrightYellow => 103,
    //         Self::BgBrightBlue => 104,
    //         Self::BgBrightMagenta => 105,
    //         Self::BgBrightCyan => 106,
    //         Self::BgBrightWhite => 107,
    //
    //         Self::Bold => 1,
    //         Self::Dim => 2,
    //         Self::Italic => 3,
    //         Self::Underline => 4,
    //         Self::Blink => 5,
    //         Self::Reverse => 7,
    //         Self::Hidden => 8,
    //         Self::Strikethrough => 9,
    //     }
    // }

    pub const fn esc(self) -> &'static str {
        match self {
            Self::Reset => "\x1b[0m",

            Self::Black => "\x1b[30m",
            Self::Red => "\x1b[31m",
            Self::Green => "\x1b[32m",
            Self::Yellow => "\x1b[33m",
            Self::Blue => "\x1b[34m",
            Self::Magenta => "\x1b[35m",
            Self::Cyan => "\x1b[36m",
            Self::White => "\x1b[37m",

            Self::BrightBlack => "\x1b[90m",
            Self::BrightRed => "\x1b[91m",
            Self::BrightGreen => "\x1b[92m",
            Self::BrightYellow => "\x1b[93m",
            Self::BrightBlue => "\x1b[94m",
            Self::BrightMagenta => "\x1b[95m",
            Self::BrightCyan => "\x1b[96m",
            Self::BrightWhite => "\x1b[97m",

            Self::BgBlack => "\x1b[40m",
            Self::BgRed => "\x1b[41m",
            Self::BgGreen => "\x1b[42m",
            Self::BgYellow => "\x1b[43m",
            Self::BgBlue => "\x1b[44m",
            Self::BgMagenta => "\x1b[45m",
            Self::BgCyan => "\x1b[46m",
            Self::BgWhite => "\x1b[47m",

            Self::BgBrightBlack => "\x1b[100m",
            Self::BgBrightRed => "\x1b[101m",
            Self::BgBrightGreen => "\x1b[102m",
            Self::BgBrightYellow => "\x1b[103m",
            Self::BgBrightBlue => "\x1b[104m",
            Self::BgBrightMagenta => "\x1b[105m",
            Self::BgBrightCyan => "\x1b[106m",
            Self::BgBrightWhite => "\x1b[107m",

            Self::Bold => "\x1b[1m",
            Self::Dim => "\x1b[2m",
            Self::Italic => "\x1b[3m",
            Self::Underline => "\x1b[4m",
            Self::Blink => "\x1b[5m",
            Self::Reverse => "\x1b[7m",
            Self::Hidden => "\x1b[8m",
            Self::Strikethrough => "\x1b[9m",
        }
    }
}

// // const SEP: char = '─';
// const H_SEP: char = '-';
// const V_SEP: char = '│';
// type Coord = (u16, u16);
// struct React {
//     p: Coord,
//     s: Coord,
// }
// struct State {}
// pub fn run() -> std::io::Result<()> {
//     let _guard = TermGuard::new()?;
//     // let t_size = size()?;
//     let mut stdout = std::io::stdout();
//
//     loop {
//         match event::read()? {
//             // Event::FocusGained => todo!(),
//             // Event::FocusLost => todo!(),
//             Event::Key(k) => match k.code {
//                 KeyCode::Backspace => todo!(),
//                 KeyCode::Enter => {
//                     write!(stdout, "\n")?;
//                     execute!(stdout, cursor::MoveToColumn(0))?;
//                     stdout.flush()?;
//                 }
//                 KeyCode::Char(c) => {
//                     if c == 'q' {
//                         break;
//                     }
//
//                     write!(stdout, "{}", c)?;
//                     stdout.flush()?;
//                 }
//                 KeyCode::Esc => todo!(),
//                 _ => {}
//             },
//             // Event::Mouse(mouse_event) => todo!(),
//             // Event::Paste(_) => todo!(),
//             Event::Resize(_, _) => todo!(),
//             _ => {}
//         }
//     }
//
//     Ok(())
// }
// struct TermGuard;
//
// impl TermGuard {
//     fn new() -> std::io::Result<Self> {
//         let mut out = stdout();
//
//         enable_raw_mode()?;
//
//         execute!(
//             out,
//             EnterAlternateScreen,
//             Clear(ClearType::All),
//             MoveTo(0, 0),
//         )?;
//
//         Ok(Self)
//     }
// }
// impl Drop for TermGuard {
//     fn drop(&mut self) {
//         let mut out = stdout();
//
//         let _ = execute!(out, LeaveAlternateScreen,);
//
//         let _ = disable_raw_mode();
//     }
// }
