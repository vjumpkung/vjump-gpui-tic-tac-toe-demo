//! UI integration tests: the production view in a headless window, driven by
//! native clicks and key presses.
//!
//! Imports are explicit on purpose. A glob import of `gpui_kit` would pull in
//! GPUI's `test` macro and shadow Rust's built-in `#[test]`.

use gpui_kit::test::TestWindowExt;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Entity, TestAppContext, WindowBounds, WindowOptions,
};

use crate::app::{self, TicTacToeApp};
use crate::game::Mark;
use crate::session::{Mode, Session};
use crate::theme;

fn open_app(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<TicTacToeApp>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        theme::install(cx);
        app::bind_keys(cx);
        let bounds = Bounds::maximized(None, cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            cx,
            |window, cx| cx.new(|cx| TicTacToeApp::new(window, cx)),
        )
        .expect("open test window")
    })
}

/// The element id of the board cell at `ix`, matching `("cell", ix)` in the view.
fn cell_id(ix: usize) -> (&'static str, usize) {
    ("cell", ix)
}

fn count_marks(session: &Session, mark: Mark) -> usize {
    (0..9)
        .filter(|&ix| session.board().get(ix) == Some(mark))
        .count()
}

#[gpui_kit::test]
fn clicking_a_cell_marks_it_and_the_bot_replies(cx: &mut TestAppContext) {
    let (window_handle, app) = open_app(cx);

    cx.update_window(window_handle, |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(window.find("status").label(), Some("Your turn"));
        assert_eq!(
            window.find(cell_id(4)).label(),
            Some("Row 2, column 2, empty")
        );

        window.click(cell_id(4), cx);

        assert_eq!(window.find("status").label(), Some("Your turn"));
        assert_eq!(window.find(cell_id(4)).label(), Some("Row 2, column 2, X"));
    })
    .unwrap();

    cx.update(|cx| {
        let session = app.read(cx).session();
        assert_eq!(session.board().get(4), Some(Mark::X));
        assert_eq!(count_marks(session, Mark::X), 1);
        assert_eq!(count_marks(session, Mark::O), 1);
    });
}

#[gpui_kit::test]
fn arrow_keys_move_cell_focus_and_enter_plays_the_cell(cx: &mut TestAppContext) {
    let (window_handle, app) = open_app(cx);

    cx.update_window(window_handle, |_, window, cx| {
        app.update(cx, |app, cx| app.focus_cell(4, window, cx));
        window.render_frame(cx);
        assert_eq!(window.find(cell_id(4)).focused(), Some(true));

        window.press("left", cx);
        assert_eq!(window.find(cell_id(3)).focused(), Some(true));

        window.press("enter", cx);
    })
    .unwrap();

    cx.update(|cx| {
        let session = app.read(cx).session();
        assert_eq!(session.board().get(3), Some(Mark::X));
        assert_eq!(count_marks(session, Mark::O), 1);
    });
}

#[gpui_kit::test]
fn switching_to_two_players_resets_the_board(cx: &mut TestAppContext) {
    let (window_handle, app) = open_app(cx);

    cx.update_window(window_handle, |_, window, cx| {
        window.render_frame(cx);
        window.click(cell_id(4), cx);
        window.click("mode-two-players", cx);

        assert_eq!(window.find("status").label(), Some("X to move"));
        assert_eq!(
            window.find(cell_id(4)).label(),
            Some("Row 2, column 2, empty")
        );
    })
    .unwrap();

    cx.update(|cx| {
        let session = app.read(cx).session();
        assert_eq!(session.mode(), Mode::TwoPlayers);
        assert!(session.board().is_empty());
    });
}

#[gpui_kit::test]
fn the_new_round_shortcut_clears_the_board(cx: &mut TestAppContext) {
    let (window_handle, app) = open_app(cx);

    cx.update_window(window_handle, |_, window, cx| {
        window.render_frame(cx);
        window.click(cell_id(4), cx);
        assert!(!app.read(cx).session().board().is_empty());

        window.press("secondary-n", cx);
    })
    .unwrap();

    cx.update(|cx| {
        assert!(app.read(cx).session().board().is_empty());
    });
}
