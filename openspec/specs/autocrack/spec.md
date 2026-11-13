# autocrack Specification

## Purpose

The attack-runner mode: pick a victim cipher, watch the cryptanalytic attack execute live step by step, and see the recovered key and plaintext — the mode every future attack demo (SWEET32, FREAK, Logjam) will plug into.

## Requirements

### Requirement: Victim cipher selection
The Autocrack screen SHALL list the available victim ciphers (at minimum Caesar, Vigenère, SWEET32, FREAK, and Logjam) with their sample ciphertexts or traffic descriptions, and SHALL let the user select one via arrow keys and start the attack with Enter (bare `1`-`4` remain the shell's mode keys).

#### Scenario: Entering the mode shows the victim list
- **WHEN** the user switches to Autocrack with no attack running
- **THEN** the screen lists every available victim cipher with a short sample of its ciphertext, one selectable per line

#### Scenario: Enter starts the selected attack
- **WHEN** the user presses Enter on a selected victim cipher
- **THEN** the attack for that cipher begins running on its built-in sample

### Requirement: Attack advances live per tick
While an attack is running, it SHALL advance one step per tick, displaying each candidate key with its fitness score as it is tried, and SHALL highlight the best-scoring candidate seen so far.

#### Scenario: Candidates and scores stream in
- **WHEN** an attack is running
- **THEN** newly tried candidates appear with their scores, and the best candidate so far is visually distinguished

#### Scenario: The attack terminates on its own
- **WHEN** an attack has tried its final candidate
- **THEN** the attack stops advancing and the screen reports completion

### Requirement: Completion shows the attack's recovered result
On completion, the screen SHALL display the attack's recovered result: the key and decrypted plaintext for cipher breaks; the colliding blocks and recovered plaintext material for birthday attacks; or the recovered structural secret and what it unlocks for factoring and discrete-log attacks — FREAK showing the modulus's primes, the derived private key, and the decrypted secret, Logjam showing the recovered exponent, the shared secret, and the decrypted session.

#### Scenario: Done state
- **WHEN** an attack completes
- **THEN** the best key and the plaintext decrypted under it are both visible

### Requirement: Reset and mode-switch safety
The Autocrack screen SHALL reset to the victim list on `r`, and switching modes mid-attack SHALL preserve attack progress until the user returns or resets.

#### Scenario: r returns to the victim list
- **WHEN** `r` is pressed in any autocrack state
- **THEN** the screen returns to victim selection with no attack running

#### Scenario: Progress survives a mode switch
- **WHEN** the user switches to another mode while an attack is running and later returns
- **THEN** the attack state is exactly as it was left

### Requirement: SWEET32 birthday attack runs to a collision
The SWEET32 victim SHALL run a birthday-collision attack over the cipher core's simulated block traffic: while running, it SHALL display blocks sent, collisions found, and the expected collision count from the birthday math, and it SHALL terminate on its own at the first collision, reporting the collision's block count against the theoretical birthday bound and the cookie material recovered by XORing the colliding records.

#### Scenario: Progress streams against the birthday expectation
- **WHEN** the SWEET32 attack is running
- **THEN** the screen shows blocks sent so far, collisions found so far, and the expected count growing with the birthday curve

#### Scenario: First collision ends the attack with the leak
- **WHEN** a collision occurs
- **THEN** the attack completes showing how many blocks were sent, the birthday-bound expectation for that block space, and the recovered cookie material

### Requirement: FREAK run streams downgrade then factoring
The FREAK victim SHALL open with the downgrade exchange as visible rows (client offering strong RSA, the man-in-the-middle stripping it to RSA_EXPORT, the server complying), then stream Pollard's rho factoring iterations against the toy export modulus tick by tick, and SHALL terminate on its own when the modulus factors, reporting total iterations, the two primes, the derived private exponent, and the decrypted secret.

#### Scenario: Downgrade plays before the attack
- **WHEN** a FREAK attack starts
- **THEN** the first streamed rows narrate the downgrade from strong RSA to export RSA before factoring begins

#### Scenario: Factoring lands and the chain completes
- **WHEN** the rho iterations find a factor
- **THEN** the attack completes showing the iteration count, both primes, the private exponent derived from them, and the secret decrypted under the recovered key

### Requirement: Logjam run streams downgrade then discrete log
The Logjam victim SHALL open with the downgrade exchange as visible rows (client offering strong DHE, the man-in-the-middle stripping it to DHE_EXPORT, the server reusing the weak group), then stream baby-step giant-step progress against the toy export group tick by tick, and SHALL terminate on its own when the discrete log is found, reporting total steps, the recovered exponent, the shared secret, and the session bytes it decrypts.

#### Scenario: Downgrade plays before the attack
- **WHEN** a Logjam attack starts
- **THEN** the first streamed rows narrate the downgrade from strong DHE to the export group before the discrete log begins

#### Scenario: The discrete log lands and the session falls
- **WHEN** BSGS finds the exponent
- **THEN** the attack completes showing the step count, the recovered exponent, the shared secret, and the decrypted session bytes

### Requirement: Victim list responds to mouse
The Autocrack screen SHALL treat a left-click on a victim row as selecting that victim and starting its attack immediately, and clicks while an attack is running SHALL be inert.

#### Scenario: Clicking a victim starts its attack
- **WHEN** the user clicks the row listing SWEET32
- **THEN** the SWEET32 attack begins running

#### Scenario: Clicks during a running attack change nothing
- **WHEN** the user clicks anywhere while an attack is running
- **THEN** the attack continues unaffected
