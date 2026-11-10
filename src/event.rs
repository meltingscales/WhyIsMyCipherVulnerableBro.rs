//! Crossterm event handling: maps key presses to app actions and provides
//! the tick heartbeat the future animated demos will draw against.

use std::io;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// What the main loop should do in response to an input event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    NextTab,
    PrevTab,
    Select(usize),
    Quit,
    /// Poll timed out with no input; the loop's animation heartbeat.
    Tick,
}

/// Map a key event to an action. Releases, repeats, and unmapped keys return
/// `None`; char actions require bare modifiers so combos like Ctrl-Q stay
/// inert while Shift+Tab (reported as `BackTab`) still cycles.
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

/// Wait up to `timeout` for the next input event. Key events map through
/// [`key_to_action`]; non-key events are ignored; a timeout yields
/// [`Action::Tick`].
pub fn next_action(timeout: Duration) -> io::Result<Option<Action>> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) => Ok(key_to_action(key)),
            _ => Ok(None),
        }
    } else {
        Ok(Some(Action::Tick))
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
}
