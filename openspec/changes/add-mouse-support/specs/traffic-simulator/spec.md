# Spec Delta

## ADDED Requirements

### Requirement: Session list responds to mouse
The Traffic Simulator screen SHALL treat a left-click on a session row as selecting that session and running its decrypt, and clicks outside the list SHALL be inert.

#### Scenario: Clicking a session selects and decrypts
- **WHEN** the user clicks the row listing login.example.com
- **THEN** that session becomes selected and its break begins

#### Scenario: Clicks outside the list do nothing
- **WHEN** the user clicks in the hex or plaintext panes
- **THEN** the selection and decrypt state are unchanged
