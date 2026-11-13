# Spec Delta

## ADDED Requirements

### Requirement: Chart clicks cycle samples
The Math Views screen SHALL treat a left-click in the left half of its chart area as cycling to the previous sample and a click in the right half as cycling to the next, in every view.

#### Scenario: Clicking the chart halves cycles samples
- **WHEN** the user clicks the right half of the chart area
- **THEN** the next sample renders; clicking the left half returns to the previous one
