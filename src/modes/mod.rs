//! Per-mode screen modules. Each mode owns its state, key handling, ticking,
//! and rendering; the shell dispatches to the active one and every other
//! mode's state persists untouched while inactive.

pub mod autocrack;
pub mod lessons;
pub mod mathviews;
pub mod traffic;
