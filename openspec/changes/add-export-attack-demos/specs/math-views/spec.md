# Spec Delta

## MODIFIED Requirements

### Requirement: Birthday-collision view
The Math Views screen SHALL offer a second view in the `v` cycle, charting the birthday attack's collision data: collisions observed as blocks accumulate against the expected birthday count for the demo block space, with both curves labeled and the view named. Pressing `v` cycles frequency → birthday → attack-cost → frequency.

#### Scenario: v toggles between views
- **WHEN** `v` is pressed on the frequency view
- **THEN** the birthday-collision view renders in its place, and `v` again moves to the attack-cost view, and once more returns to the frequency view

#### Scenario: Birthday view shows actual against expected
- **WHEN** the birthday view is displayed
- **THEN** the chart shows the observed collision count and the expected birthday curve over blocks sent, both labeled

## ADDED Requirements

### Requirement: Attack-cost view
The Math Views screen SHALL offer a third view in the `v` cycle, charting attack cost as observed solver steps against the expected complexity curves at several problem sizes: Pollard's rho factoring and baby-step giant-step discrete log near the √N / N^¼ expectations, against brute force's N, all series labeled with the view named.

#### Scenario: Cost curves render for every solver
- **WHEN** the attack-cost view is displayed
- **THEN** the chart shows observed and expected step counts for both solvers across the problem sizes, each labeled
