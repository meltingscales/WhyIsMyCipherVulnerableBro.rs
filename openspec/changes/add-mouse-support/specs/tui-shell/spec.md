# Spec Delta

## MODIFIED Requirements

### Requirement: Safe terminal lifecycle
The application SHALL enter an alternate-screen raw-mode terminal with mouse capture enabled on startup, and SHALL restore the user's original terminal state — including disabling mouse capture — on every exit path, including panics.

#### Scenario: Clean exit restores the terminal
- **WHEN** the user quits the application normally
- **THEN** the original terminal state (main screen, cooked mode, cursor visible, mouse capture off) is restored before the process exits

#### Scenario: Panic restores the terminal
- **WHEN** the application panics while running
- **THEN** the terminal state is restored, mouse capture disabled, before the panic message is shown, leaving the user's shell usable

## ADDED Requirements

### Requirement: Tab bar responds to mouse
The application SHALL treat a left-click on a tab-bar tab as a direct switch to that mode, before any other click routing, and SHALL route clicks outside the tab bar to the active mode.

#### Scenario: Clicking a tab switches modes
- **WHEN** the user left-clicks the tab showing "Lessons"
- **THEN** Lessons becomes the active mode

#### Scenario: Clicks below the tab bar reach the active mode
- **WHEN** the user clicks inside the screen body while any mode is active
- **THEN** the active mode receives the click and inactive modes do not
