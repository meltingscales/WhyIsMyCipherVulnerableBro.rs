# Spec Delta

## MODIFIED Requirements

### Requirement: Shipped lessons cover the implemented attacks
The screen SHALL ship with at least five lessons matching implemented attacks: Caesar falling to frequency analysis, breaking repeating-XOR, SWEET32's birthday problem, FREAK's factoring of export RSA, and Logjam's discrete log against export Diffie–Hellman.

#### Scenario: Both starter lessons are present
- **WHEN** the lesson list is displayed
- **THEN** lessons on the Caesar frequency attack, repeating-XOR, the SWEET32 birthday attack, FREAK, and Logjam are all listed and openable
