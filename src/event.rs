//! Crossterm event handling: maps key presses to app actions and provides
//! the tick heartbeat the animated demos draw against.
//!
//! Global keys (mode switching, quit) are claimed before any mode sees them;
//! every other key press routes to the active mode's handler.

use std::io;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// A resolved global action for the main loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    NextTab,
    PrevTab,
    Select(usize),
    Quit,
}

/// One event for the main loop: a resolved global action, a key press for the
/// active mode to interpret, or the animation heartbeat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    Global(Action),
    ModeKey(KeyEvent),
    Tick,
}

/// Where a key press belongs: the global shell or the active mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyClass {
    Global,
    Mode,
}

/// Map a key event to a global action. Releases, repeats, and unmapped keys
/// return `None`; char actions require bare modifiers so combos like Ctrl-Q
/// stay inert while Shift+Tab (reported as `BackTab`) still cycles.
pub fn key_to_action(key: KeyEvent) -> Option<Action> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    match (key.code, key.modifiers) {
        (KeyCode::Tab, _) => Some(Action::NextTab),
        (KeyCode::BackTab, _) => Some(Action::PrevTab),
        (KeyCode::Char(digit @ '1'..='4'), KeyModifiers::NONE) => {
            Some(Action::Select(digit as usize - '1' as usize))
        }
        (KeyCode::Char('q'), KeyModifiers::NONE) => Some(Action::Quit),
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Action::Quit),
        _ => None,
    }
}

/// Classify a key press: global keys never reach a mode, mode keys only reach
/// the active one. Non-press events are ignored entirely.
pub fn classify(key: KeyEvent) -> Option<KeyClass> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    if key_to_action(key).is_some() {
        Some(KeyClass::Global)
    } else {
        Some(KeyClass::Mode)
    }
}

/// Wait up to `timeout` for the next input event: global keys resolve to
/// their action, other key presses pass through for the active mode,
/// non-key events are ignored, and a timeout yields [`AppEvent::Tick`].
pub fn next_app_event(timeout: Duration) -> io::Result<Option<AppEvent>> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) => match classify(key) {
                Some(KeyClass::Global) => Ok(Some(AppEvent::Global(
                    key_to_action(key).expect("classified keys map to actions"),
                ))),
                Some(KeyClass::Mode) => Ok(Some(AppEvent::ModeKey(key))),
                None => Ok(None),
            },
            _ => Ok(None),
        }
    } else {
        Ok(Some(AppEvent::Tick))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn tab_moves_forward() {
        assert_eq!(
            key_to_action(press(KeyCode::Tab, KeyModifiers::NONE)),
            Some(Action::NextTab)
        );
    }

    #[test]
    fn backtab_moves_backward() {
        // Shift+Tab arrives as BackTab, usually with SHIFT set.
        assert_eq!(
            key_to_action(press(KeyCode::BackTab, KeyModifiers::SHIFT)),
            Some(Action::PrevTab)
        );
    }

    #[test]
    fn number_keys_select_modes() {
        for (digit, index) in [('1', 0), ('2', 1), ('3', 2), ('4', 3)] {
            assert_eq!(
                key_to_action(press(KeyCode::Char(digit), KeyModifiers::NONE)),
                Some(Action::Select(index))
            );
        }
    }

    #[test]
    fn q_quits() {
        assert_eq!(
            key_to_action(press(KeyCode::Char('q'), KeyModifiers::NONE)),
            Some(Action::Quit)
        );
    }

    #[test]
    fn ctrl_c_quits() {
        assert_eq!(
            key_to_action(press(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Some(Action::Quit)
        );
    }

    #[test]
    fn unmapped_keys_are_ignored() {
        assert_eq!(
            key_to_action(press(KeyCode::Char('x'), KeyModifiers::NONE)),
            None
        );
        assert_eq!(key_to_action(press(KeyCode::Esc, KeyModifiers::NONE)), None);
        assert_eq!(
            key_to_action(press(KeyCode::Char('Q'), KeyModifiers::SHIFT)),
            None
        );
    }

    #[test]
    fn key_releases_are_ignored() {
        let release = KeyEvent::new_with_kind(
            KeyCode::Char('q'),
            KeyModifiers::NONE,
            KeyEventKind::Release,
        );
        assert_eq!(key_to_action(release), None);
    }

    #[test]
    fn global_keys_classify_global_and_never_reach_a_mode() {
        for key in [
            press(KeyCode::Tab, KeyModifiers::NONE),
            press(KeyCode::BackTab, KeyModifiers::SHIFT),
            press(KeyCode::Char('3'), KeyModifiers::NONE),
            press(KeyCode::Char('q'), KeyModifiers::NONE),
            press(KeyCode::Char('c'), KeyModifiers::CONTROL),
        ] {
            assert_eq!(classify(key), Some(KeyClass::Global), "{key:?}");
        }
    }

    #[test]
    fn mode_keys_classify_mode() {
        for key in [
            press(KeyCode::Enter, KeyModifiers::NONE),
            press(KeyCode::Up, KeyModifiers::NONE),
            press(KeyCode::Down, KeyModifiers::NONE),
            press(KeyCode::Left, KeyModifiers::NONE),
            press(KeyCode::Right, KeyModifiers::NONE),
            press(KeyCode::Esc, KeyModifiers::NONE),
            press(KeyCode::Char('r'), KeyModifiers::NONE),
        ] {
            assert_eq!(classify(key), Some(KeyClass::Mode), "{key:?}");
        }
    }

    #[test]
    fn releases_do_not_classify() {
        let release =
            KeyEvent::new_with_kind(KeyCode::Enter, KeyModifiers::NONE, KeyEventKind::Release);
        assert_eq!(classify(release), None);
    }
}
