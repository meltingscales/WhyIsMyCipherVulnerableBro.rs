# Spec Delta

## MODIFIED Requirements

### Requirement: Victim cipher selection
The Autocrack screen SHALL list the available victim ciphers (at minimum Caesar, Vigenère, SWEET32, FREAK, and Logjam) with their sample ciphertexts or traffic descriptions, and SHALL let the user select one via arrow keys and start the attack with Enter (bare `1`-`4` remain the shell's mode keys).

#### Scenario: Entering the mode shows the victim list
- **WHEN** the user switches to Autocrack with no attack running
- **THEN** the screen lists every available victim cipher, one selectable per line, each with a short sample of its ciphertext or traffic

#### Scenario: Enter starts the selected attack
- **WHEN** the user presses Enter on a selected victim cipher
- **THEN** the attack for that cipher begins running on its built-in sample

### Requirement: Completion shows the attack's recovered result
On completion, the screen SHALL display the attack's recovered result: the key and decrypted plaintext for cipher breaks; the colliding blocks and recovered plaintext material for birthday attacks; or the recovered structural secret and what it unlocks for factoring and discrete-log attacks — FREAK showing the modulus's primes, the derived private key, and the decrypted secret, Logjam showing the recovered exponent, the shared secret, and the decrypted session.

#### Scenario: Done state
- **WHEN** an attack completes
- **THEN** the attack's recovered result — key and plaintext, collision and leaked material, or structural secret and unlocked plaintext — is visible

## ADDED Requirements

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
