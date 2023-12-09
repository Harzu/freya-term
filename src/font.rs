use alacritty_terminal::ansi::NamedColor;
use skia_safe::Color;

pub const FONT_FILE_PATH: &str = "./fonts/Hack Regular Nerd Font Complete.ttf";
pub const FONT_SIZE: f32 = 20.0;

pub fn get_color(c: alacritty_terminal::ansi::Color) -> Color {
    match c {
        alacritty_terminal::ansi::Color::Spec(rgb) => Color::from_rgb(rgb.r, rgb.g, rgb.b),
        alacritty_terminal::ansi::Color::Named(c) => match c {
            NamedColor::Foreground => Color::from_rgb(235, 218, 177),
            NamedColor::Background => Color::from_rgb(40, 39, 39),
            NamedColor::Green => Color::from_rgb(152, 150, 26),
            NamedColor::Red => Color::from_rgb(203, 35, 29),
            NamedColor::Yellow => Color::from_rgb(214, 152, 33),
            NamedColor::Blue => Color::from_rgb(69, 132, 135),
            NamedColor::Cyan => Color::from_rgb(104, 156, 105),
            NamedColor::Magenta => Color::from_rgb(176, 97, 133),
            NamedColor::White => Color::from_rgb(168, 152, 131),
            NamedColor::Black => Color::from_rgb(40, 39, 39),
            NamedColor::BrightBlack => Color::from_rgb(146, 130, 115),
            NamedColor::BrightRed => Color::from_rgb(250, 72, 52),
            NamedColor::BrightGreen => Color::from_rgb(184, 186, 38),
            NamedColor::BrightYellow => Color::from_rgb(249, 188, 47),
            NamedColor::BrightBlue => Color::from_rgb(131, 164, 151),
            NamedColor::BrightMagenta => Color::from_rgb(210, 133, 154),
            NamedColor::BrightCyan => Color::from_rgb(142, 191, 123),
            NamedColor::BrightWhite => Color::from_rgb(235, 218, 177),
            NamedColor::BrightForeground => Color::from_rgb(235, 218, 177),
            _ => Color::from_rgb(40, 39, 39),
        },
        _ => Color::from_rgb(40, 39, 39),
    }
}
