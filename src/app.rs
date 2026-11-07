//! The Tic Tac Toe window: header, mode chips, status line, board and scores.
//!
//! `TicTacToeApp` owns the `Session` (domain state) plus the focus handles.
//! `render` only reads that state; every change goes through a method that
//! notifies once.

use std::rc::Rc;

use gpui_kit::component::{
    ActiveTheme, Disableable, IconName, Selectable, StyledExt, Theme, ThemeMode, button::Button,
    h_flex, v_flex,
};
use gpui_kit::prelude::*;
use gpui_kit::{
    Action, App, FocusHandle, KeyBinding, KeyBindingContextPredicate, Role, SharedString,
    TestSupportExt, Window, div, relative, rems,
};

use crate::game::{Mark, Outcome};
use crate::session::{Mode, Session, Status};
use crate::theme::{Brand, brand};

gpui_kit::actions!(
    tic_tac_toe,
    [NewRound, FocusUp, FocusDown, FocusLeft, FocusRight]
);

const APP_CONTEXT: &str = "TicTacToe";
const BOARD_CONTEXT: &str = "Board";
/// The cell that holds the roving Tab stop until the player moves it.
const CENTER_CELL: usize = 4;

/// Registers the keyboard shortcuts: Ctrl+N (Cmd+N on macOS) starts a new
/// round, and the arrow keys move between cells while the board has focus.
pub fn bind_keys(cx: &mut App) {
    let bindings = [
        key_binding("secondary-n", NewRound, APP_CONTEXT, cx),
        key_binding("up", FocusUp, BOARD_CONTEXT, cx),
        key_binding("down", FocusDown, BOARD_CONTEXT, cx),
        key_binding("left", FocusLeft, BOARD_CONTEXT, cx),
        key_binding("right", FocusRight, BOARD_CONTEXT, cx),
    ];
    cx.bind_keys(bindings);
}

/// `KeyBinding::new` only takes explicit modifiers; `load` resolves
/// `secondary-` to Ctrl or Cmd for the host OS.
fn key_binding(keystrokes: &str, action: impl Action, context: &str, cx: &App) -> KeyBinding {
    let predicate = KeyBindingContextPredicate::parse(context).expect("valid key context");
    KeyBinding::load(
        keystrokes,
        Box::new(action),
        Some(Rc::new(predicate)),
        false,
        None,
        cx,
    )
    .expect("valid keystrokes")
}

/// What the status line shows for a state: an optional mark tag, the sentence
/// beside it, and the full text announced to assistive technology.
struct StatusCopy {
    tag: Option<Mark>,
    text: &'static str,
    label: SharedString,
}

fn status_copy(mode: Mode, status: Status) -> StatusCopy {
    let with_tag = |mark: Mark, text: &'static str| StatusCopy {
        tag: Some(mark),
        text,
        label: format!("{} {text}", mark.as_str()).into(),
    };
    let plain = |text: &'static str| StatusCopy {
        tag: None,
        text,
        label: text.into(),
    };
    let tagged_plain = |mark: Mark, text: &'static str| StatusCopy {
        tag: Some(mark),
        text,
        label: text.into(),
    };
    match (mode, status) {
        (Mode::OnePlayer, Status::Turn(Mark::X)) => tagged_plain(Mark::X, "Your turn"),
        (Mode::OnePlayer, Status::Won(Mark::X)) => tagged_plain(Mark::X, "You win"),
        (Mode::OnePlayer, Status::Won(Mark::O)) => tagged_plain(Mark::O, "Bot wins"),
        (_, Status::Draw) => plain("Draw"),
        (_, Status::Turn(mark)) => with_tag(mark, "to move"),
        (_, Status::Won(mark)) => with_tag(mark, "wins"),
    }
}

pub struct TicTacToeApp {
    session: Session,
    /// Focus for the whole window, so the new-round shortcut works at launch.
    focus_handle: FocusHandle,
    /// One handle per cell. Only the handle at `cursor` is a Tab stop, so Tab
    /// enters the board once and the arrow keys move within it.
    cell_focus: [FocusHandle; 9],
    cursor: usize,
}

impl TicTacToeApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        let cell_focus = std::array::from_fn(|ix| cx.focus_handle().tab_stop(ix == CENTER_CELL));
        Self {
            session: Session::new(Mode::OnePlayer),
            focus_handle,
            cell_focus,
            cursor: CENTER_CELL,
        }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Moves the roving Tab stop to `ix`. Returns whether it moved.
    ///
    /// A handle passed to `track_focus` keeps its own tab settings, so the
    /// stop is switched on the handles themselves rather than on the button.
    fn set_cursor(&mut self, ix: usize) -> bool {
        if ix == self.cursor {
            return false;
        }
        let previous = self.cursor;
        self.cell_focus[previous] = self.cell_focus[previous].clone().tab_stop(false);
        self.cell_focus[ix] = self.cell_focus[ix].clone().tab_stop(true);
        self.cursor = ix;
        true
    }

    fn play_cell(&mut self, ix: usize, cx: &mut Context<Self>) {
        let moved = self.set_cursor(ix);
        let played = self.session.play(ix);
        if moved || played {
            cx.notify();
        }
    }

    fn set_mode(&mut self, mode: Mode, cx: &mut Context<Self>) {
        if self.session.set_mode(mode) {
            cx.notify();
        }
    }

    fn new_round(&mut self, cx: &mut Context<Self>) {
        if self.session.new_round() {
            cx.notify();
        }
    }

    fn toggle_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let next = if cx.theme().is_dark() {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };
        // `Theme::change` reloads the brand theme and refreshes every window.
        Theme::change(next, Some(window), cx);
    }

    /// Moves the cursor to cell `ix` and gives it keyboard focus.
    pub fn focus_cell(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let moved = self.set_cursor(ix);
        window.focus(&self.cell_focus[ix], cx);
        if moved {
            cx.notify();
        }
    }

    /// Moves the cursor by whole rows and columns, stopping at the board edge.
    fn move_cursor(
        &mut self,
        row_step: isize,
        column_step: isize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let row = (self.cursor / 3).saturating_add_signed(row_step).min(2);
        let column = (self.cursor % 3).saturating_add_signed(column_step).min(2);
        self.focus_cell(row * 3 + column, window, cx);
    }

    fn on_new_round(&mut self, _: &NewRound, _: &mut Window, cx: &mut Context<Self>) {
        self.new_round(cx);
    }

    fn on_focus_up(&mut self, _: &FocusUp, window: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor(-1, 0, window, cx);
    }

    fn on_focus_down(&mut self, _: &FocusDown, window: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor(1, 0, window, cx);
    }

    fn on_focus_left(&mut self, _: &FocusLeft, window: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor(0, -1, window, cx);
    }

    fn on_focus_right(&mut self, _: &FocusRight, window: &mut Window, cx: &mut Context<Self>) {
        self.move_cursor(0, 1, window, cx);
    }

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_dark = theme.is_dark();
        let radius_full = theme.radius_full();
        let brand = brand(cx);
        let (icon, tooltip) = if is_dark {
            (IconName::Sun, "Switch to light theme")
        } else {
            (IconName::Moon, "Switch to dark theme")
        };
        // The marker is a swipe behind the title on paper and a thick
        // underline on the dark desk, where a pale swipe would wash out the
        // light text.
        let title = if is_dark {
            div()
                .text_2xl()
                .font_bold()
                .text_decoration_2()
                .text_decoration_color(brand.highlight_yellow)
                .child("Tic Tac Toe")
        } else {
            div()
                .relative()
                .text_2xl()
                .font_bold()
                .child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .bottom(relative(0.08))
                        .h(relative(0.42))
                        .bg(brand.highlight_yellow),
                )
                .child("Tic Tac Toe")
        };
        h_flex().justify_between().child(title).child(
            Button::new("theme-toggle")
                .icon(icon)
                .rounded(radius_full)
                .size_10()
                .tooltip(tooltip)
                .accessibility_label(tooltip)
                .on_click(cx.listener(|this, _, window, cx| this.toggle_theme(window, cx))),
        )
    }

    fn render_mode_chips(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let radius_full = theme.radius_full();
        let (foreground, background) = (theme.foreground, theme.background);
        let current_mode = self.session().mode();
        let chips = [
            ("mode-one-player", "1 player", Mode::OnePlayer),
            ("mode-two-players", "2 players", Mode::TwoPlayers),
        ];
        h_flex().gap_2().children(chips.map(|(id, label, mode)| {
            let is_current = mode == current_mode;
            Button::new(id)
                .label(label)
                .rounded(radius_full)
                .h_10()
                .px_4()
                .font_medium()
                .selected(is_current)
                .toggled(is_current)
                .when(is_current, |this| {
                    // The active filter chip inverts ink and paper.
                    this.bg(foreground)
                        .text_color(background)
                        .border_color(foreground)
                })
                .on_click(cx.listener(move |this, _, _, cx| this.set_mode(mode, cx)))
        }))
    }

    fn render_status(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_dark = theme.is_dark();
        let radius_full = theme.radius_full();
        let transparent = theme.transparent;
        let brand = brand(cx);
        let session = self.session();
        let copy = status_copy(session.mode(), session.status());
        let tag_border = if is_dark { transparent } else { brand.ink };
        h_flex()
            .justify_between()
            .h_10()
            .child(
                div()
                    .id("status")
                    .role(Role::Status)
                    .test_support()
                    .aria_label(copy.label)
                    .child(
                        h_flex()
                            .gap_2()
                            .when_some(copy.tag, |this, mark| {
                                let fill = match mark {
                                    Mark::X => brand.highlight_pink,
                                    Mark::O => brand.highlight_cyan,
                                };
                                this.child(
                                    div()
                                        .px_2p5()
                                        .py_0p5()
                                        .rounded(radius_full)
                                        .text_xs()
                                        .font_semibold()
                                        .text_color(brand.ink)
                                        .border_1()
                                        .border_color(tag_border)
                                        .bg(fill)
                                        .child(mark.as_str()),
                                )
                            })
                            .child(div().text_lg().font_semibold().child(copy.text)),
                    ),
            )
            .child(
                Button::new("new-round")
                    .label("New round")
                    .rounded(radius_full)
                    .h_10()
                    .px_4()
                    .font_medium()
                    .disabled(session.board().is_empty())
                    .tooltip_with_action("New round", &NewRound, Some(APP_CONTEXT))
                    .on_click(cx.listener(|this, _, _, cx| this.new_round(cx))),
            )
    }

    fn render_cell(&self, ix: usize, brand: Brand, cx: &mut Context<Self>) -> Button {
        let theme = cx.theme();
        let (radius_lg, foreground) = (theme.radius_lg, theme.foreground);
        let session = self.session();
        let mark = session.board().get(ix);
        let is_winning = matches!(
            session.outcome(),
            Outcome::Won { line, .. } if line.contains(&ix)
        );
        let glyph_color = if is_winning {
            brand.ink
        } else if mark == Some(Mark::X) {
            brand.pink_ink
        } else {
            foreground
        };
        let state = mark.map_or("empty", Mark::as_str);
        let button = Button::new(("cell", ix))
            .track_focus(&self.cell_focus[ix])
            .size_24()
            .rounded(radius_lg)
            .bg(if is_winning {
                brand.highlight_yellow
            } else {
                brand.card
            })
            .accessibility_label(format!(
                "Row {}, column {}, {state}",
                ix / 3 + 1,
                ix % 3 + 1
            ))
            .when_some(mark, |this, mark| {
                this.child(
                    div()
                        .text_size(rems(3.))
                        .font_bold()
                        .text_color(glyph_color)
                        .child(mark.as_str()),
                )
            });
        if session.is_playable(ix) {
            button.on_click(cx.listener(move |this, _, _, cx| this.play_cell(ix, cx)))
        } else {
            // selected() suppresses hover/pressed feedback on unplayable cells;
            // never use disabled() here because disabled buttons leave the Tab
            // order and would break board keyboard focus.
            button.selected(true)
        }
    }

    fn render_board(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let brand = brand(cx);
        let mut board = v_flex()
            .gap_4()
            .key_context(BOARD_CONTEXT)
            .on_action(cx.listener(Self::on_focus_up))
            .on_action(cx.listener(Self::on_focus_down))
            .on_action(cx.listener(Self::on_focus_left))
            .on_action(cx.listener(Self::on_focus_right));
        for row in 0..3 {
            let mut cells = h_flex().gap_4();
            for column in 0..3 {
                cells = cells.child(self.render_cell(row * 3 + column, brand, cx));
            }
            board = board.child(cells);
        }
        board
    }

    fn render_scores(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let (radius_lg, border, muted_foreground) =
            (theme.radius_lg, theme.border, theme.muted_foreground);
        let card = brand(cx).card;
        let session = self.session();
        let score = session.score();
        let (x_label, o_label) = match session.mode() {
            Mode::OnePlayer => ("You (X)", "Bot (O)"),
            Mode::TwoPlayers => ("Player X", "Player O"),
        };
        let tiles = [
            (x_label, score.x_wins()),
            ("Draws", score.draws()),
            (o_label, score.o_wins()),
        ];
        h_flex().gap_3().children(tiles.map(|(label, count)| {
            v_flex()
                .flex_1()
                .p_4()
                .rounded(radius_lg)
                .border_1()
                .border_color(border)
                .bg(card)
                .child(
                    div()
                        .text_sm()
                        .text_color(muted_foreground)
                        .font_medium()
                        .child(label),
                )
                .child(div().text_2xl().font_bold().child(count.to_string()))
        }))
    }
}

impl Render for TicTacToeApp {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let (background, foreground) = (theme.background, theme.foreground);
        div()
            .track_focus(&self.focus_handle)
            .key_context(APP_CONTEXT)
            .on_action(cx.listener(Self::on_new_round))
            .size_full()
            .bg(background)
            .text_color(foreground)
            .flex()
            .items_center()
            .justify_center()
            .child(
                v_flex()
                    .w_80()
                    .gap_6()
                    .child(self.render_header(cx))
                    .child(self.render_mode_chips(cx))
                    .child(self.render_status(cx))
                    .child(self.render_board(cx))
                    .child(self.render_scores(cx)),
            )
    }
}
