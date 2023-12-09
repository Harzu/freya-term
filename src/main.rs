mod terminal;
mod font;

use freya::{
    prelude::*,
    events::keyboard::Key,
};
use skia_safe::{Rect, Paint, Font, Data, Typeface, TextBlob};
use std::time::Duration;
use futures::StreamExt;
use tokio::{fs::File, select};
use tokio::io::AsyncReadExt;
use tokio::time::sleep;
use tokio::sync::mpsc::unbounded_channel;

use crate::font::{FONT_FILE_PATH, FONT_SIZE};

const DEFAULT_WIDTH: f32 = 800.0;
const DEFAULT_HEIGHT: f32 = 600.0;
const DPI_SCALE_FACTOR: f32 = 2.0;

fn main() {
    launch_cfg(
        app,
        LaunchConfig::<()>::builder()
            .with_background("rgb(40, 39, 39)")
            .with_width(DEFAULT_WIDTH as f64)
            .with_height(DEFAULT_HEIGHT as f64)
            .with_title("Terminal app")
            .build(),
    );
}

enum FrontendAction {
    TermResize((f32, f32))
}

fn app(cx: Scope) -> Element {
    let mut terminal = terminal::Terminal::new("/bin/bash".to_string());
    let reader = terminal.new_reader();
    let font_data_raw = std::fs::read(FONT_FILE_PATH).expect("Failed to read font file");
    let font = Font::new(
        Typeface::from_data(
            Data::new_copy(&font_data_raw),
            None
        ).unwrap(),
        FONT_SIZE,
    );
    let font_measure = font.measure_str("W", Some(&Paint::default()));
    let font_height = font_measure.1.height();
    let font_witdth = font_measure.1.width();

    let term = use_ref(cx, || terminal);
    let state = use_state(cx, || vec![]);
    let window_size = use_state(cx, || (DEFAULT_WIDTH, DEFAULT_HEIGHT));
    let (size_tx, mut size_rx) = unbounded_channel();

    cx.use_hook(|| {
        to_owned![term, state];
        cx.spawn(async move {
            let local_reader = reader.try_clone().unwrap();
            let mut file = File::from(local_reader);

            loop {
                let mut buf = [0; 4096];
                if let Ok(_) = file.read(&mut buf).await {
                    term.write().update(buf.to_vec());
                    let cells = term.read().cells();
                    state.set(cells);
                };
                sleep(Duration::from_millis(1)).await;
            }
        });
    });

    cx.use_hook(|| {
        to_owned![term, font_height, font_witdth, window_size];
        cx.spawn(async move {
            loop {
                let sleep = sleep(Duration::from_millis(100));
                tokio::pin!(sleep);
                select! {
                    size = size_rx.recv() => {
                        window_size.set(size.unwrap());
                    },
                    _ = &mut sleep => {
                        let row_count = ((window_size.current().1 as f32) / font_height).round() as u16;
                        let col_count = 2 * ((window_size.current().0 as f32) / font_witdth).round() as u16;
                        term.write().resize(row_count, col_count);
                    },
                };
            }
        });
    });

    let front_chan = use_coroutine(cx, |mut rx: UnboundedReceiver<FrontendAction>| {
        async move {
            while let Some(action) = rx.next().await {
                match action {
                    FrontendAction::TermResize(size) => {
                        size_tx.send(size).expect("send new term size is failed");
                    }
                }
            }
        }
    });

    let canvas = use_canvas(cx, state, |state| {
        to_owned![front_chan, font, font_height, font_witdth];
        let cells = state.get().clone();
        Box::new(move |canvas, _, region| {
            front_chan.send(FrontendAction::TermResize((
                region.width() / DPI_SCALE_FACTOR,
                region.height() / DPI_SCALE_FACTOR
            )));

            canvas.save();
            for cell in cells.clone() {
                let x = cell.column as f32 * font_witdth;
                let cell_line = cell.line + cell.display_offset as i32;
                let mut text_paint = Paint::default();
                text_paint.set_anti_alias(true);
                text_paint.set_color(cell.fg);

                let text_blob = TextBlob::from_str(&cell.content.to_string(), &font).expect("Failed to create TextBlob");
                let text_blob_width = text_blob.bounds().width();
                let text_blob_heigh = text_blob.bounds().height();
                let rect = Rect::from_xywh(
                    x,
                    cell_line as f32 * font_height * 2.0,
                    text_blob_width,
                    text_blob_heigh
                );
                let mut bg_paint = Paint::default();
                bg_paint.set_color(cell.bg);
                bg_paint.set_anti_alias(true);

                canvas.draw_rect(&rect, &bg_paint);        
                canvas.draw_text_blob(
                    text_blob,
                    (
                        x,
                        (cell_line as f32 + 0.7) * font_height * 2.0
                    ),
                    &text_paint
                );
            }
            canvas.restore();
        })
    });

    let onkeydown = move |e: KeyboardEvent| {
        to_owned![term];
        match &e.key {
            Key::Character(c) => {
                term.write().write_to_pty(c.chars().next().unwrap());
            },
            Key::Enter => {
                term.write().write_to_pty('\n');
            }
            _ => {},
        }
    };

    render!(
        rect {
            height: "100%",
            width: "100%",
            onkeydown: onkeydown,
            Canvas {
                canvas: canvas,
                background: "rgb(40, 39, 39)",
                width: "100%",
                height: "100%"
            }
        }
    )
}
