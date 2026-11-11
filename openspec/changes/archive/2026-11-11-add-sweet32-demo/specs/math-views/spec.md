# Spec Delta

## ADDED Requirements

### Requirement: Birthday-collision view
The Math Views screen SHALL offer a second view, toggled with `v`, charting the birthday attack's collision data: collisions observed as blocks accumulate against the expected birthday count for the demo block space, with both curves labeled and the view named.

#### Scenario: v toggles between views
- **WHEN** `v` is pressed on the frequency view
- **THEN** the birthday-collision view renders in its place, and `v` again returns to the frequency view

#### Scenario: Birthday view shows actual against expected
- **WHEN** the birthday view is displayed
- **THEN** the chart shows the observed collision count and the expected birthday curve over blocks sent, both labeled
