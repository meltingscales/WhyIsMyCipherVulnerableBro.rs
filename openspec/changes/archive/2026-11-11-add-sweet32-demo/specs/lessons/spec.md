# Spec Delta

## MODIFIED Requirements

### Requirement: Shipped lessons cover the implemented attacks
The screen SHALL ship with at least three lessons matching implemented attacks: Caesar falling to frequency analysis, breaking repeating-XOR, and SWEET32 — the birthday problem against small block ciphers.

#### Scenario: Both starter lessons are present
- **WHEN** the lesson list is displayed
- **THEN** lessons on the Caesar frequency attack, on repeating-XOR, and on the SWEET32 birthday attack are all listed and openable
