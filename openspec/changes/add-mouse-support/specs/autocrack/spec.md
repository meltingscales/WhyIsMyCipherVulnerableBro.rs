# Spec Delta

## ADDED Requirements

### Requirement: Victim list responds to mouse
The Autocrack screen SHALL treat a left-click on a victim row as selecting that victim and starting its attack immediately, and clicks while an attack is running SHALL be inert.

#### Scenario: Clicking a victim starts its attack
- **WHEN** the user clicks the row listing SWEET32
- **THEN** the SWEET32 attack begins running

#### Scenario: Clicks during a running attack change nothing
- **WHEN** the user clicks anywhere while an attack is running
- **THEN** the attack continues unaffected
