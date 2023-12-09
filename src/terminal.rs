use alacritty_terminal::{
    ansi::{self, Processor},
    event::{EventListener, OnResize, WindowSize},
    term::{test::TermSize, cell::Flags},
    tty::{EventedReadWrite, Pty},
    Term,
};
use skia_safe::Color;
use std::{
    fs::File,
    io::Write,
};

use crate::font;

#[derive(Clone)]
pub struct EventProxy;

impl EventProxy {}

impl EventListener for EventProxy {
    fn send_event(&self, _: alacritty_terminal::event::Event) {}
}

pub struct Terminal {
    tty: Pty,
    term: Term<EventProxy>,
    parser: Processor,
}

impl Terminal {
    pub fn new(shell: String) -> Self {
        let mut config = alacritty_terminal::config::Config::default();

        config.pty_config.shell = Some(alacritty_terminal::config::Program::WithArgs {
            program: shell,
            args: vec![],
        });

        let size = WindowSize {
            cell_width: 1,
            cell_height: 1,
            num_cols: 100,
            num_lines: 50,
        };
        let term_size = TermSize::new(100, 50);
        let event_proxy = EventProxy {};
        let tty = alacritty_terminal::tty::new(&config.pty_config, size, 0).unwrap();
        let term = alacritty_terminal::Term::new(&config, &term_size, event_proxy);
        let parser = ansi::Processor::new();

        Self { tty, term, parser }
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        let size = WindowSize {
            cell_width: 1,
            cell_height: 1,
            num_cols: cols,
            num_lines: rows,
        };

        self.tty.on_resize(size);
        self.term.resize(TermSize::new(
            size.num_cols as usize,
            size.num_lines as usize,
        ));
    }

    pub fn new_reader(&mut self) -> File {
        self.tty.reader().try_clone().unwrap()
    }

    pub fn update(&mut self, data: Vec<u8>) {
        for item in data.to_vec() {
            self.parser.advance(&mut self.term, item);
        }
    }

    pub fn write_to_pty(&mut self, c: char) {
        self.tty.writer().write_all(&[c as u8]).unwrap();
    }

    pub fn cells(&self) -> Vec<Cell> {
        let mut res = vec![];
        let content = self.term.renderable_content();
        for item in content.display_iter {
                let point = item.point;
                let cell = item.cell;
                let mut fg = font::get_color(cell.fg);
                let mut bg = font::get_color(cell.bg);

                if cell.flags.contains(Flags::DIM) || cell.flags.contains(Flags::DIM_BOLD) {
                    fg = Color::from_argb(66, fg.r(), fg.g(), fg.b());
                }

                let inverse = cell.flags.contains(Flags::INVERSE);
                if inverse {
                    let clone_fg = fg.clone();
                    fg = bg;
                    bg = clone_fg;
                }

                res.push(Cell {
                    column: point.column.0,
                    line: point.line.0,
                    content: cell.c,
                    display_offset: content.display_offset,
                    fg: fg,
                    bg: bg,
                })
        }

        res
    }
}

#[derive(Clone, Debug)]
pub struct Cell {
    pub column: usize,
    pub line: i32,
    pub content: char,
    pub display_offset: usize,
    pub fg: Color,
    pub bg: Color,
}
