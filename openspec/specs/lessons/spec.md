# lessons Specification

## Purpose

The guided-reading mode: pick a lesson and walk through it step by step, cryptopals-style — each lesson explains the math behind one of the attacks the app can run, in the order you would discover it.

## Requirements

### Requirement: Lesson list
The Lessons screen SHALL list the available lessons, selectable via arrow keys or number keys, and opening one SHALL display it starting at its first step.

#### Scenario: Entering the mode shows the lesson list
- **WHEN** the user switches to Lessons with no lesson open
- **THEN** the screen lists every available lesson by title, one selectable per line

#### Scenario: Enter opens the selected lesson at step one
- **WHEN** the user presses Enter on a lesson
- **THEN** the lesson's first step is displayed with its step counter

### Requirement: Step navigation
Within an open lesson, the screen SHALL move forward one step on right-arrow/`n` and back one step on left-arrow/`p`, clamped at both ends, and SHALL display the current step number and total.

#### Scenario: Stepping forward and back
- **WHEN** a lesson is open and right-arrow is pressed
- **THEN** the next step is displayed and the step counter advances

#### Scenario: Steps clamp at the ends
- **WHEN** right-arrow is pressed on the last step, or left-arrow on the first
- **THEN** the displayed step does not change

### Requirement: Escape returns to the lesson list
The screen SHALL return to the lesson list, closing the open lesson, on Escape or `r`.

#### Scenario: Escape closes the lesson
- **WHEN** Escape is pressed while a lesson is open
- **THEN** the screen returns to the lesson list with no lesson open

### Requirement: Shipped lessons cover the implemented attacks
The screen SHALL ship with at least five lessons matching implemented attacks: Caesar falling to frequency analysis, breaking repeating-XOR, SWEET32's birthday problem, FREAK's factoring of export RSA, and Logjam's discrete log against export Diffie–Hellman.

#### Scenario: Both starter lessons are present
- **WHEN** the lesson list is displayed
- **THEN** lessons on the Caesar frequency attack, repeating-XOR, the SWEET32 birthday attack, FREAK, and Logjam are all listed and openable

### Requirement: Lesson list responds to mouse
The Lessons screen SHALL treat a left-click on a lesson row as opening that lesson at its first step, and clicks while a lesson is open SHALL be inert.

#### Scenario: Clicking a lesson opens it
- **WHEN** the user clicks the row listing the SWEET32 lesson
- **THEN** that lesson opens at step one

#### Scenario: Clicks inside an open lesson change nothing
- **WHEN** the user clicks anywhere while a lesson is open
- **THEN** the displayed step is unchanged
