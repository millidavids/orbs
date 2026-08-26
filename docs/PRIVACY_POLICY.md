# Privacy Policy

*Draft — pending legal review.*

**Effective date:** 2026-08-26
**Last updated:** 2026-08-26

## Summary

O.R.B.S. is developed by Blackhearth Studios. We do not operate any servers that
collect your personal data. We do not use third-party analytics, advertising, or
tracking services. **The game has no networking code of any kind** — it cannot
connect to anything, including us. All of your game data stays on your own
computer, with the exception of anything you choose to share through Steam's
platform features.

## What Data the Game Stores Locally

O.R.B.S. saves your progress to a single human-readable TOML file. It contains
the state of your tower: what you have brewed, learned, and built, where you are
standing, the spells you have written, and the seed and command history that
reproduce your session.

**In this build the save is written beside the game's executable, as
`orbs-save.toml`.** Moving it into your operating system's standard application
data directory arrives with the settings screen, alongside Steam Cloud support.
When that lands, this policy will be updated with the new locations and the
"Last updated" date above will change.

You can set the environment variable `ORBS_SAVE` to a path of your choosing to
put the file elsewhere, or to `off` to disable saving entirely for that session.

You can delete the save file at any time to remove all locally stored data.
Doing so will reset your progress.

### Spell files

Scripts you write inside the game are stored within the same save file. They are
your text, kept verbatim — the game never rewrites a spell you have written.

### Diagnostic files the game can write

Two files are written **only when you explicitly ask for them**, and neither is
ever transmitted anywhere:

- **`orbs-parse.tsv`** — press `F6` to write a trace of how the parser
  interpreted your commands. This is a debugging aid for bug reports.
- **`orbs-screenshot.png`** — press `F12` to capture the screen.

Both are written beside the game's executable. Delete them freely.

## Steam Platform Data

If you play O.R.B.S. through Steam, your play session may interact with Valve's
Steamworks APIs to support achievements, statistics, and — if you enable it —
Steam Cloud save syncing.

This data is handled by Valve under Valve's own privacy policy, available at
<https://store.steampowered.com/privacy_agreement/>. Blackhearth Studios does not
receive personally identifying information from Valve — we see only anonymous
aggregate statistics.

> **Note for this build:** Steamworks integration is **not yet present** in the
> game. No achievements, stats, or Cloud syncing are active. This section
> describes what the shipping build will do and will be revised to match it.

## No Networking, No Multiplayer, No Telemetry

O.R.B.S. is a single-player game with **no network functionality whatsoever**.
There is no multiplayer, no leaderboard, no matchmaking, and no online component
of any kind. The game contains no HTTP client, no socket code, and no async
runtime; this is enforced as an architectural rule in the codebase rather than
being merely a current state of affairs.

The game does not use Google Analytics, crash reporters, advertising SDKs, or
any other third-party telemetry service. **The game does not phone home, and
could not if it wanted to.**

## Children's Privacy

O.R.B.S. is not directed at children under 13. We do not knowingly collect
personal information from children. Because we do not collect personal
information from anyone, no special data-handling procedures for children are
required.

## Your Rights (GDPR / CCPA)

Under the EU General Data Protection Regulation and the California Consumer
Privacy Act, you have the right to:

- **Access** the personal data held about you
- **Correct** inaccurate personal data
- **Delete** personal data held about you
- **Object to processing** of your personal data
- **Data portability**

Because all of your O.R.B.S. data is stored locally on your own computer — in a
plain-text file you can open in any editor — you can exercise all of these
rights yourself by reading, editing, or deleting the save file described above.
Blackhearth Studios does not hold any copy of your personal data that you would
need to contact us to retrieve or erase.

## Changes to This Policy

We may update this policy from time to time. The "Last updated" date above will
reflect the most recent change. Material changes will be announced in the game's
changelog and on the O.R.B.S. website.

## Contact

If you have questions about this privacy policy, contact us at
**support@blackhearthgames.com**.
