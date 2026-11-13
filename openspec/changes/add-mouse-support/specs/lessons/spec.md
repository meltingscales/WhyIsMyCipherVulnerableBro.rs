# Spec Delta

## ADDED Requirements

### Requirement: Lesson list responds to mouse
The Lessons screen SHALL treat a left-click on a lesson row as opening that lesson at its first step, and clicks while a lesson is open SHALL be inert.

#### Scenario: Clicking a lesson opens it
- **WHEN** the user clicks the row listing the SWEET32 lesson
- **THEN** that lesson opens at step one

#### Scenario: Clicks inside an open lesson change nothing
- **WHEN** the user clicks anywhere while a lesson is open
- **THEN** the displayed step is unchanged
