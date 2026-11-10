//! Thin entry point: owns terminal lifecycle and the app loop.

mod app;
mod event;
mod ui;

use std::time::Duration;

use crate::app::App;
use crate::event::{Action, next_action};

/// Animation heartbeat; the loop redraws at most this often between inputs.
const TICK: Duration = Duration::from_millis(250);

fn main() -> std::io::Result<()> {
    // init() enters the alternate screen, enables raw mode, and installs a
    // panic hook that restores the terminal before the panic message prints.
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
    let mut app = App::new();
    loop {
        terminal.draw(|frame| ui::render(frame, &app))?;
        match next_action(TICK)? {
            Some(Action::NextTab) => app.next(),
            Some(Action::PrevTab) => app.prev(),
            Some(Action::Select(index)) => app.select(index),
            Some(Action::Quit) => app.quit(),
            Some(Action::Tick) | None => {}
        }
        if app.should_quit {
            return Ok(());
        }
    }
}
