#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod bot;
mod game;
mod session;
mod theme;
#[cfg(test)]
mod ui_tests;

use gpui_kit::{AppContext as _, TitlebarOptions, WindowBounds, WindowOptions, px, size};

fn main() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx| {
            gpui_kit::init(cx);
            theme::install(cx);
            app::bind_keys(cx);
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();

            // Window geometry is a platform boundary, so it is the one place
            // that names pixels.
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::centered(size(px(480.), px(760.)), cx)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Tic Tac Toe".into()),
                    ..Default::default()
                }),
                window_min_size: Some(size(px(400.), px(720.))),
                ..Default::default()
            };
            gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| app::TicTacToeApp::new(window, cx))
            })
            .expect("failed to open window");
            cx.activate(true);
        });
}
