//! Thin entry point: owns terminal lifecycle and the app loop.
//! `--mcp` swaps the terminal for the headless harness server.

mod app;
mod cipher;
mod event;
mod mcp;
mod modes;
mod ui;

use std::io::{self, stdout};
use std::time::Duration;

use ratatui::crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use ratatui::crossterm::execute;

use crate::app::App;
use crate::event::{Action, AppEvent, next_app_event};

/// Animation heartbeat; the loop redraws at most this often between inputs.
const TICK: Duration = Duration::from_millis(250);

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    match (args.next().as_deref(), args.next()) {
        (Some("--mcp"), None) => mcp::server::serve(),
        (Some("--mcp"), Some(extra)) => {
            eprintln!("unexpected argument after --mcp: {extra}");
            std::process::exit(2);
        }
        (Some(other), _) => {
            eprintln!(
                "unknown argument: {other}\nusage: {} [--mcp]",
                std::env::args().next().unwrap_or_default()
            );
            std::process::exit(2);
        }
        (None, _) => run_tui(),
    }
}

fn run_tui() -> std::io::Result<()> {
    // init() enters the alternate screen, enables raw mode, and installs a
    // panic hook that restores the terminal before the panic message prints.
    let mut terminal = ratatui::init();
    // ratatui's hook restores raw mode and the alternate screen but knows
    // nothing about mouse capture: chain ours in front so a panic can never
    // leave the user's terminal with capture stuck on.
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(stdout(), DisableMouseCapture);
        previous_hook(info);
    }));
    execute!(stdout(), EnableMouseCapture)?;
    let result = run(&mut terminal);
    restore_terminal();
    result
}

/// Undo everything the TUI did to the terminal, mouse capture included.
fn restore_terminal() {
    let _ = execute!(stdout(), DisableMouseCapture);
    ratatui::restore();
}

fn run(terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
    let mut app = App::new();
    loop {
        terminal.draw(|frame| ui::render(frame, &mut app))?;
        match next_app_event(TICK)? {
            Some(AppEvent::Global(Action::NextTab)) => app.next(),
            Some(AppEvent::Global(Action::PrevTab)) => app.prev(),
            Some(AppEvent::Global(Action::Select(index))) => app.select(index),
            Some(AppEvent::Global(Action::Quit)) => app.quit(),
            Some(AppEvent::ModeKey(key)) => app.mode_key(key),
            Some(AppEvent::MouseClick { column, row }) => {
                app.mouse_click(ratatui::layout::Position { x: column, y: row });
            }
            Some(AppEvent::Tick) => app.tick(),
            None => {}
        }
        if app.should_quit {
            return Ok(());
        }
    }
}
