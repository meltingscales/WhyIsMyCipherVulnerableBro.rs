# Spec Delta

## Purpose

The interception mode: browse a list of fake intercepted sessions (all traffic is simulated locally — nothing touches a network), inspect a session's ciphertext as hex, then run the break and watch the plaintext reveal.

## ADDED Requirements

### Requirement: Fake session list
The Traffic Simulator screen SHALL present a list of built-in simulated intercepted sessions, each showing at least a fake hostname, a cipher label, and a length, selectable via arrow keys or number keys.

#### Scenario: Entering the mode shows sessions
- **WHEN** the user switches to Traffic Simulator
- **THEN** the screen lists every built-in session with its hostname, cipher label, and ciphertext length, one selectable per line

### Requirement: Hex inspection of the selected session
The screen SHALL show a hex dump of the selected session's ciphertext alongside the session list, updating as the selection moves.

#### Scenario: Selecting a session shows its bytes
- **WHEN** the user selects a session
- **THEN** a hex dump of that session's ciphertext is displayed next to the list

### Requirement: Decrypt action reveals plaintext progressively
On Enter, the screen SHALL run the session's break (repeating-XOR against known plaintext structure) and reveal the recovered plaintext progressively per tick until fully shown.

#### Scenario: Enter starts the reveal
- **WHEN** the user presses Enter on a session that has not been decrypted yet
- **THEN** the recovered plaintext begins revealing progressively on subsequent ticks

#### Scenario: Fully revealed plaintext stays visible
- **WHEN** the reveal has finished
- **THEN** the complete recovered plaintext remains displayed for that session

### Requirement: Reset and mode-switch safety
The screen SHALL reset all session state on `r`, and switching modes SHALL preserve session states until the user returns or resets.

#### Scenario: r clears all decryption state
- **WHEN** `r` is pressed on the Traffic Simulator screen
- **THEN** every session returns to undecrypted and the selection resets

#### Scenario: Session state survives a mode switch
- **WHEN** the user switches modes and returns
- **THEN** selection and decryption progress are exactly as left
