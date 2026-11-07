# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Desktop Tic Tac Toe (1 player vs bot, 2 players) in Rust + GPUI via the `gpui-kit` 0.9.0 crate, styled with the vjump.dev "Highlighter Notebook" design system. See README.md for features, controls and the design-system mapping.

## Commands

```sh
cargo run                                   # debug build
cargo run --release                         # release exe has no console window (windows_subsystem)
cargo test                                  # all tests (pure + UI integration)
cargo test the_bot_never_loses              # one test by name substring
cargo test session::tests                   # one module's tests
cargo test ui_tests                         # only the GPUI UI integration tests
cargo clippy --all-targets -- -D warnings   # must stay clean
cargo fmt --check
```

- Debug, test (`test-support` feature) and release each compile GPUI separately; a cold build takes several minutes. Use long timeouts or run in the background.
- Windows needs the MSVC toolchain and the Windows SDK (`rc.exe` embeds the manifest; release builds also use `fxc.exe`, overridable with `GPUI_FXC_PATH`).

## Architecture

Two layers with a strict dependency direction:

- **Pure core (no GPUI imports):** `game.rs` (Mark, Board, Outcome, win lines) ← `bot.rs` (deterministic minimax, lowest index wins ties) ← `session.rs` (Mode, Score, Status, play flow). Everything game-related is decided here and tested with plain `#[test]`.
- **GPUI shell:** `app.rs` (`TicTacToeApp` view, actions, key bindings), `theme.rs` (fonts, theme registration, brand colors), `main.rs` (bootstrap and window).

Key behaviors that span files:

- `Session` is the single source of truth. Its mutators (`play`, `new_round`, `set_mode`) return `bool` = "something changed", and the view calls `cx.notify()` only when it is true. `render` never mutates.
- In `Mode::OnePlayer`, `Session::play` applies the human move (always X, always first) **and** the bot's O reply synchronously in the same call. There is no async, timer or "thinking" state by design. The score is recorded exactly once per finished round inside `Session`.
- Keyboard: the root div holds `focus_handle` (focused in `TicTacToeApp::new` so Ctrl+N works at launch) with key context `"TicTacToe"`; the board has key context `"Board"` for the arrow actions. The 9 cell `FocusHandle`s form a roving Tab stop: `set_cursor` flips `FocusHandle::tab_stop` on the stored handles, because `Button::tab_stop` has no effect on a handle passed via `track_focus`.
- Key bindings go through the `key_binding()` helper using `KeyBinding::load`, because `KeyBinding::new` does not resolve `secondary-` (Ctrl on Windows/Linux, Cmd on macOS) and panics.
- Cells are `Button`s. Unplayable cells get `.selected(true)` and no `on_click` (suppresses hover/pressed feedback). **Never use `.disabled()` on cells**: disabled buttons leave the Tab order and break board focus.
- UI tests address elements by stable ids and read accessibility labels: `"status"` (`Role::Status`, label such as `"Your turn"` / `"X to move"`), cells `("cell", ix)` labelled `"Row r, column c, X|O|empty"`, `"mode-one-player"`, `"mode-two-players"`, `"new-round"`, `"theme-toggle"`. Changing copy or ids means updating `ui_tests.rs`.

## Theme and styling rules

- **Raw colors live only in `themes/vjump.json`.** No hex/`rgb`/`hsla` literals in `src/`. Both "Vjump Light" and "Vjump Dark" are installed as gpui-kit's `light_theme`/`dark_theme`, so `Theme::change` (the toggle) and system sync keep the palette. `Theme::change` reloads the registered theme and overwrites ad-hoc `Theme::update` color edits, so never patch colors in code.
- Brand roles without a dedicated gpui-kit field are read by `theme::brand()` from existing `ThemeColor` fields: card ← `group_box`, ink ← `primary_foreground`, pink ink ← `ring`, yellow ← `primary`, cyan ← `cyan`, pink ← `magenta`. A new color role means a new JSON key (see `ThemeConfigColors` in gpui-component's `theme/schema.rs`) read through the theme, not a literal.
- `px(...)` is allowed only in `main.rs` (window geometry). Elsewhere use rem-based helpers (`h_10`, `size_24`, `gap_4`, `rems(..)`).
- GPUI has no `text_4xl`/`text_5xl`; use `.text_size(rems(..))`. `Button::label` forces `text_sm`, so large glyphs are a child `div`. Button has no 40px size tier; 40px pills use instance styling (`.h_10().px_4()` / `.size_10()` with `.rounded(theme.radius_full())`).
- Design source: `c:\Users\vjump\Documents\webdev\vjump-dev-new-website\DESIGN.md`. Highlighter fills (yellow/cyan/pink) always carry ink text; highlighters are never text on the light theme (use pink ink). Copy is sentence case with no trailing periods. Accepted deviations are listed in README.md.

## gpui-kit 0.9.0 notes

- Depend only on `gpui-kit` (it re-exports GPUI, pinned as `gpui-pre` 0.3.11). Do not add Zed's `gpui` crate.
- The `gpui-kit` and `gpui-kit-design-guides` skills hold the normative coding and design guides, but the skill's `usage.md` is from 0.7 and partly stale (e.g. `cx.theme().surface` / `.destructive` and a `card` field do not exist on `ThemeColor`). Verify signatures in the cached 0.9.0 source under `~/.cargo/registry/src/index.crates.io-*/` (`gpui-kit-0.9.0`, `gpui-component-0.9.0`, `gpui-base-0.9.0`, `gpui-pre-0.3.11`) instead of guessing.

## Testing

- Pure tests live inline (`#[cfg(test)] mod tests`) in `game.rs`, `bot.rs`, `session.rs`, including an exhaustive search proving the bot never loses and every round is scored once.
- `src/ui_tests.rs` uses `#[gpui_kit::test]` with `TestAppContext` and `gpui_kit::test::TestWindowExt` (`render_frame`, `find(id).label()` / `.focused()`, `click`, `press`). It runs on Windows (no pixel rendering). Keep imports explicit: never `use gpui_kit::*` or `use super::*` there, since GPUI's `test` macro would shadow `#[test]`.
