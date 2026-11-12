# math-views Specification

## Purpose

The visualization mode: the math behind the attacks drawn as data — frequency histograms and the statistics (chi-squared, index of coincidence) that make classical breaks work, rendered from the same cipher core the demos run on.

## Requirements

### Requirement: Letter-frequency chart with sample switching
The Math Views screen SHALL display a letter-frequency bar chart of the currently selected sample and SHALL switch samples on left/right-arrow, cycling through at least an English plaintext sample and Caesar- and Vigenère-encrypted ciphertext samples.

#### Scenario: Chart renders for every sample
- **WHEN** the user switches to Math Views
- **THEN** a letter-frequency bar chart of the current sample is displayed with the sample named

#### Scenario: Arrows cycle the samples
- **WHEN** right-arrow is pressed
- **THEN** the chart re-renders for the next sample, wrapping from the last to the first; left-arrow cycles backward

### Requirement: Fitness statistics accompany the chart
For the displayed sample, the screen SHALL show the chi-squared distance from English letter frequencies and the index of coincidence.

#### Scenario: Statistics update with the sample
- **WHEN** the user switches samples
- **THEN** the displayed chi-squared value and index of coincidence update to match the new sample

#### Scenario: Plaintext scores better than ciphertext
- **WHEN** the plaintext sample and a ciphertext sample are displayed in turn
- **THEN** the plaintext sample shows the smaller chi-squared distance and an index of coincidence nearer English

### Requirement: Birthday-collision view
The Math Views screen SHALL offer a second view in the `v` cycle, charting the birthday attack's collision data: collisions observed as blocks accumulate against the expected birthday count for the demo block space, with both curves labeled and the view named. Pressing `v` cycles frequency → birthday → attack-cost → frequency.

#### Scenario: v toggles between views
- **WHEN** `v` is pressed on the frequency view
- **THEN** the birthday-collision view renders in its place, and `v` again moves to the attack-cost view, and once more returns to the frequency view

#### Scenario: Birthday view shows actual against expected
- **WHEN** the birthday view is displayed
- **THEN** the chart shows the observed collision count and the expected birthday curve over blocks sent, both labeled

### Requirement: Attack-cost view
The Math Views screen SHALL offer a third view in the `v` cycle, charting attack cost as observed solver steps against the expected complexity curves at several problem sizes: Pollard's rho factoring and baby-step giant-step discrete log near the √N / N^¼ expectations, against brute force's N, all series labeled with the view named.

#### Scenario: Cost curves render for every solver
- **WHEN** the attack-cost view is displayed
- **THEN** the chart shows observed and expected step counts for both solvers across the problem sizes, each labeled
