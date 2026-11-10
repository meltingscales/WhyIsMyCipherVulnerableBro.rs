# Spec Delta

## Purpose

The visualization mode: the math behind the attacks drawn as data — frequency histograms and the statistics (chi-squared, index of coincidence) that make classical breaks work, rendered from the same cipher core the demos run on.

## ADDED Requirements

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
