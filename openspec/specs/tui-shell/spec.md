# tui-shell Specification

## Purpose

The terminal application shell that every mode screen plugs into: it owns the event loop, safe terminal lifecycle, navigation between modes, the tab bar, and the footer keybinding help.

## Requirements

### Requirement: Safe terminal lifecycle
The application SHALL enter an alternate-screen raw-mode terminal on startup and SHALL restore the user's original terminal state on every exit path, including panics.

#### Scenario: Clean exit restores the terminal
- **WHEN** the user quits the application normally
- **THEN** the original terminal state (main screen, cooked mode, cursor visible) is restored before the process exits

#### Scenario: Panic restores the terminal
- **WHEN** the application panics while running
- **THEN** the terminal state is restored before the panic message is shown, leaving the user's shell usable

### Requirement: Mode navigation
The application SHALL provide exactly four modes — Autocrack, Traffic Simulator, Lessons, Math Views — and SHALL switch the active mode on Tab (next), Shift+Tab (previous), and direct selection keys 1-4, with wrap-around at both ends.

#### Scenario: Tab cycles forward
- **WHEN** Autocrack is active and Tab is pressed
- **THEN** Traffic Simulator becomes the active mode

#### Scenario: Tab wraps from the last mode to the first
- **WHEN** Math Views is active and Tab is pressed
- **THEN** Autocrack becomes the active mode

#### Scenario: Shift+Tab cycles backward with wrap-around
- **WHEN** Autocrack is active and Shift+Tab is pressed
- **THEN** Math Views becomes the active mode

#### Scenario: Number key selects a mode directly
- **WHEN** the key `2` is pressed in any mode
- **THEN** Traffic Simulator becomes the active mode

### Requirement: Tab bar identifies the active mode
The application SHALL display a persistent tab bar listing all four modes and SHALL visually distinguish the active mode in it.

#### Scenario: Active tab is highlighted
- **WHEN** a mode becomes active
- **THEN** the tab bar highlights that mode's tab and the screen body renders that mode's content

### Requirement: Footer shows global keybindings
The application SHALL display a persistent footer listing the global keybindings (mode switching and quit) and SHALL quit on `q` and on Ctrl-C.

#### Scenario: Quit key exits the application
- **WHEN** `q` is pressed in any mode
- **THEN** the application exits via the clean-exit path

#### Scenario: Footer lists the essential bindings
- **WHEN** the application is running
- **THEN** the footer shows the mode-switch and quit keybindings

### Requirement: Placeholder mode screens
Each mode SHALL render an identifiable placeholder screen showing the mode's name and its planned purpose, so navigation is verifiable before any mode is implemented.

#### Scenario: Placeholder identifies the mode
- **WHEN** the user navigates to any mode
- **THEN** the screen body shows that mode's name and a not-yet-implemented placeholder description
