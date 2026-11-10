# Spec Delta

## RENAMED Requirements

- FROM: `### Requirement: Completion shows key and plaintext`
- TO: `### Requirement: Completion shows the attack's recovered result`

## MODIFIED Requirements

### Requirement: Victim cipher selection
The Autocrack screen SHALL list the available victim ciphers (at minimum Caesar, Vigenère, and SWEET32) with their sample ciphertexts or traffic descriptions, and SHALL let the user select one via arrow keys and start the attack with Enter (bare `1`-`4` remain the shell's mode keys).

#### Scenario: Entering the mode shows the victim list
- **WHEN** the user switches to Autocrack with no attack running
- **THEN** the screen lists every available victim cipher, one selectable per line, each with a short sample of its ciphertext or traffic

#### Scenario: Enter starts the selected attack
- **WHEN** the user presses Enter on a selected victim cipher
- **THEN** the attack for that cipher begins running on its built-in sample

### Requirement: Completion shows the attack's recovered result
On completion, the screen SHALL display the attack's recovered result: the key and decrypted plaintext for cipher breaks, or the colliding blocks and the plaintext material recovered from them for birthday attacks.

#### Scenario: Done state
- **WHEN** an attack completes
- **THEN** the attack's recovered result — key and plaintext, or collision and leaked material — is visible

## ADDED Requirements

### Requirement: SWEET32 birthday attack runs to a collision
The SWEET32 victim SHALL run a birthday-collision attack over the cipher core's simulated block traffic: while running, it SHALL display blocks sent, collisions found, and the expected collision count from the birthday math, and it SHALL terminate on its own at the first collision, reporting the collision's block count against the theoretical birthday bound and the cookie material recovered by XORing the colliding records.

#### Scenario: Progress streams against the birthday expectation
- **WHEN** the SWEET32 attack is running
- **THEN** the screen shows blocks sent so far, collisions found so far, and the expected count growing with the birthday curve

#### Scenario: First collision ends the attack with the leak
- **WHEN** a collision occurs
- **THEN** the attack completes showing how many blocks were sent, the birthday-bound expectation for that block space, and the recovered cookie material
