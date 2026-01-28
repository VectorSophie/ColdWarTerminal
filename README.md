# Cold War Terminal

> *The year is 198x. The world hangs by a thread. You are the operator.*

**Cold War Terminal** is an immersive CLI-based geopolitical thriller where you play as the shift operator for a high-level nuclear defense system. Your job is to analyze incoming intelligence cables, manage advisor loyalty, and prevent (or ensure) the end of the world.


## Transmission Incoming...

You have been granted **Level 5 Security Clearance**.
Your console connects directly to the **Strategic Defense Initiative (SDI)** mainframe.
You will receive encrypted cables, advisor reports, and intercept signals.
Make the right calls. The fate of 4 billion lives rests on your keystrokes.

## Gameplay Mechanics

### 1. The Dashboard
Your terminal displays real-time metrics of the geopolitical climate:
*   **DEFCON**: The closeness to nuclear launch. (1 = War, 5 = Peace)
*   **Domestic Stability**: The mood of the populace. Low stability leads to coups.
*   **System Status**: Health of the bunker's life support and computing systems.
*   **Intel Assets**: Currency used for decryption, tracing, and consulting advisors.

### 2. Directives (Commands)
You issue commands to the mainframe to resolve crises.
*   `investigate` / `inv`: Root out moles and increase weapon progress. Lowers secrecy.
*   `contain` / `con`: Attempt diplomatic de-escalation. Risks looking weak.
*   `escalate` / `esc`: Show force. Increases tension but scares the enemy.
*   `leak`: Release truth to the public. Boosts stability, lowers secrecy.
*   `decrypt [ID]`: Spend Intel to reveal encrypted content.
*   `trace`: Spend Intel to hunt for the mole interfering with signals.
*   `interrogate [NAME]`: Aggressively question an advisor (Costs 2 Intel). High risk, but may force the mole to slip up.

### 3. The Advisors (Trust No One)
Three advisors guide you. **One is a traitor.**
*   **Gen. Vance**: Military hawk. Prefers escalation.
*   **Director K.**: Intelligence spook. Obsessed with secrecy.
*   **Amb. Sterling**: Diplomat. Prefers talk over action.

Use `consult [NAME]` to get their take on the situation. Cross-reference their advice with the outcome to find the mole.

### 4. The Basilisk (System Corruption)
A hidden subroutine in the code. As you advance the secret weapon, the system's **Corruption** level rises.
*   **Anomalies**: If corruption gets too high, the AI will begin to override your commands.
*   **Autonomy**: The terminal may decide that "Peace" is inefficient and force Escalation or Purges against your will.
*   **The Secret Weapon**: It is not what you think it is.

## Installation & Running

Ensure you have [Rust](https://www.rust-lang.org/) installed.

```bash
# Clone the repository
git clone https://github.com/yourusername/cold-war-terminal.git

# Enter the bunker
cd cold-war-terminal

# Boot the system
cargo run
```

## Known Issues
*   Screen tearing may occur during high-tension events (Intentional).
*   The "Red Phone" may ring even when disconnected.
*   Do not stare directly into the ASCII art.

---
*Authorized by the Department of Defense. Unauthorized access is treason.*
