---
name: game-release
description: Fold player-facing notes into the open version block on dev (default), consolidate that block (with `consolidate` argument), or promote the gated dev tip to main (with `main` argument)
user-invocable: true
---

# Release

Three modes, by argument:

- **No argument** → *dev mode*. Fold everything unreleased — uncommitted work
  plus any commits that never opened a changelog block — into the **open version
  block** at the top of `docs/CHANGELOG.md`, then commit and push to
  `origin/dev`. Opens the next version first if the current one has shipped.
- **`consolidate`** → rewrite the open block into a clean, minimal set of
  bullets, merging overlapping entries and dropping ones that cancel out.
- **`main`** → *promotion*. Fast-forward `main` onto the gated `dev` tip and push
  it. **Changes no files and bumps nothing.**

Adapted from `court_wizard`'s skill of the same name. The branching strategy is
kept whole; the Steam half is commented out in the workflows rather than deleted,
and the parts of this skill that only made sense with Steam are marked where they
are missing.

## Invoking this skill *is* the approval

`CLAUDE.md` says never commit or push without explicit approval. **Running
`/game-release` is that approval, for whichever mode was asked for.** Dev and
consolidate modes commit and push to `origin/dev`; `main` mode pushes `main`,
which tags, releases and announces. None of them asks again.

Running a command whose documented purpose is "promote to main and announce" and
then asking whether to promote to main and announce is a report generator, not a
skill. **The typed argument is the decision** — `main` is not a word anyone
reaches for by accident, and B1's seven preconditions are the real safeguard,
each a hard stop that fires before anything leaves the machine.

This was scoped to `dev` only for one revision, on the grounds that `main` is
public and irreversible. That is true and it is not a reason to ask twice; it is
a reason for the preconditions to be strict. They are.

Everything else in `CLAUDE.md`'s git rules still binds — specific paths only, no
`git add -A`, no force-push, no attribution in the message. **Stop and ask if a
precondition fails**, never to confirm an instruction already given.

## The version is assigned on dev, never at promotion

In court_wizard this is forced: `docs/CHANGELOG.md` is `include_str!`d into the
binary, so any changelog edit changes the shipped bits and needs a rebuild — and
a rebuild at promotion time would ship a binary nobody play-tested.

**Nothing compiles the changelog into orbs**, so that argument does not apply.
The rule is kept anyway, for a weaker but sufficient reason: `main` is a
fast-forward, so the commit that gets tagged is exactly the commit
`dev-release.yml` gated. Editing a file during promotion would create a commit
`dev` has never seen and CI has never run, and the tag would point at it.

So there is no `[pending]` block and no separate lock step. At any moment the top
block of the changelog is the **open version**: already numbered, already dated.
Promotion just tags it.

### Which version is open

`git fetch --tags`, then read `V` from `Cargo.toml`
(`grep '^version = ' Cargo.toml | head -1`). The tag decides:

- **Tag `vV` does not exist** → `V` is open. Append bullets to the existing
  `## [vV] - <date>` block and refresh its date to today.
- **Tag `vV` exists** → `V` has shipped. Open the next one:
  1. Bump the patch in `Cargo.toml` (keep bumping until you find an untagged
     version). **If every patch on this minor is tagged, the minor is finished
     — ask which feature the next minor names**, and open `0.<next>.0`. The minor
     names the large feature in hand (DESIGN.md §15); it is not derived from a
     count, so it is never inferred here.
  2. `cargo update --workspace --offline` so `Cargo.lock` follows.
  3. Insert a fresh `## [vV+1] - <today>` block at the top of the changelog,
     below the `# Changelog` header **and its HTML comment**.

If tag `vV` is missing but the top block's heading names a different version,
repair it in place: rename that heading to `## [vV] - <today>` and keep its
bullets. Never stack two blocks for the same version.

**A caution specific to this project.** `CLAUDE.md` bumps the version on every
roadmap *item*, and an item is much smaller than a release. So `Cargo.toml` will
often have moved on its own between releases, and the version you are opening a
block for is whatever it says now — do not bump again on top of that. The bump in
step 2 above is only for the case where the current version is already tagged.

**The date means "last touched", not "release day".** Every dev push refreshes
it; promotion never touches it.

## How CI is wired

- **`dev-release.yml` runs on every push to `dev`.** It is CLAUDE.md's gate —
  fmt, clippy, test, doc, `cargo build -p orbs`, and the `screens` example — and
  nothing else. Cheap, minutes, no artifacts.
- **`release.yml` runs on push to `main`.** It reads the version, **requires a
  matching changelog block or does nothing at all**, runs the gate again, tags,
  publishes the GitHub Release from the block, then posts to Discord and Bluesky.

Two consequences worth stating:

- **A version with no changelog block is never released.** The workflow ends
  green having skipped everything, and says so in its summary. This is what lets
  ordinary step bumps reach `main` without announcing.
- **The gate runs twice.** court_wizard avoids that by reusing dev's artifacts;
  with nothing to reuse, the second run is what stops a promotion shipping a
  commit that only looked green. It comes out when Steam goes in.

*Missing until Steam returns:* there is no build on a staging channel, so there
is nothing to play-test between dev and main, and promotion is not gated on a
human having done so. That is the real cost of the commented-out half.

## Commit message format (dev and consolidate modes)

Release commits describe **what shipped**, not the mechanics of shipping.

```
v<version>: <lower-case summary of what this push adds>

- <one condensed line per changelog bullet this commit introduces>
```

- Subject names the version, then what changed, in the user's language. Under
  ~72 characters.
- The body lists the bullets **this commit** introduces — not the whole block. A
  later push on the same version lists only its own additions.
- If the push also contains code with no player-visible effect, do not invent a
  bullet for it.
- **No `Co-Authored-By`, no generated-with footer, no session URL.** `CLAUDE.md`
  forbids all attribution outright.

## Changelog bullet format (all modes)

- Layman's terms. No file paths, no type names, no § references — the
  engineering account belongs in `DESIGN.md` §19, which is where this project
  already puts it.
- Bold the first phrase as a short summary, then plain language.
- Only `### Description`, `### Added`, `### Changed`, `### Fixed`. Never
  `[Unreleased]`.

## The `### Description` section (all modes)

One short paragraph of plain prose — no bullet, no bold lead-in.

**It is published verbatim**: `scripts/post_to_bluesky.py` uses it as the whole
body of the Bluesky post, and it opens the Discord embed.

### Before 1.0 it is a dev log

**Say the game is in development, and write it as a log of work rather than as
patch notes.** The post goes out to people who cannot play this yet; patch notes
for a game nobody can buy announce a product that does not exist. Someone
arriving cold should be able to tell from the first clause that this is a thing
being built.

```markdown
### Description
In development — a dev log, not patch notes. The orb became a proper 4:3
monitor: the grid is fixed now, so resizing scales the text instead of reflowing
every pane. The stacks got properly random, too.
```

*What got worked on*, in a builder's voice — two or three things, most
interesting first. The bullets below carry the detail. A version that touched one
area says so in one sentence.

**This switches at 1.0**, in the same commit as the version scheme
(`0.<feature>.<iteration>` → ordinary semver, DESIGN.md §15). After it, the Description
is the release's public hook written for someone who has never heard of the game,
and the dev-log framing goes.

Three hard constraints, all of which have already been violated once:

- **It must be followed by another `### ` section.** The script reads from the
  Description heading until the next one, so a block whose bullets sit directly
  under it posts the bullets too, truncated mid-sentence.
- **~250 characters.** The limit is 300 including the `O.R.B.S. v<version>`
  title and the `Website · Studio · Source` footer, which cost ~42 between them.
- **Nothing else in `docs/CHANGELOG.md` may begin a line with `## [`** — no
  fenced examples. The format rules live in `docs/SETUP.md` §4 for exactly this
  reason; a sample in the changelog was once matched as the newest release.

Check before committing — no credentials needed:

```bash
python3 scripts/post_to_bluesky.py --version <V> --dry-run
```

**Dev mode:** write it when opening a version; on a later push to the same
version, rewrite it so it still describes the release as a whole.
**Consolidate mode:** redo it when the bullets are redone.

---

## Mode A — Dev release (no argument)

### A1. Preconditions and scope

1. `git rev-parse --abbrev-ref HEAD` must be `dev`. If not, stop and say so.
2. `git fetch --tags`.
3. Work out what is unreleased. It is **not** only the working tree — commits
   already on `dev` that never opened a block are unreleased too:
   ```bash
   git status --porcelain
   git log --oneline "$(git log -1 --format=%H -- docs/CHANGELOG.md)"..HEAD
   ```
   Uncommitted → `git diff HEAD` is the source for bullets. Committed-but-unnoted
   → `git diff <last-changelog-commit>..HEAD`. Both → cover both. **Neither** →
   stop, nothing to release.

### A2. Resolve the open version

Apply *Which version is open*. Either `Cargo.toml` is untouched, or the patch is
bumped and `Cargo.lock` updated.

### A3. Update the changelog

1. Read the top of `docs/CHANGELOG.md`.
2. Translate the diffs from A1 into player-facing bullets.
3. Append under the appropriate `### Added` / `### Changed` / `### Fixed`,
   creating the subsection if missing. Do not duplicate existing bullets.
4. Refresh the block's date to today.

**If the change has no player-facing effect at all** — CI config, developer
docs, a skill, tooling not compiled into the game — there is no honest bullet.
Do not invent one and do not edit the changelog. The change goes to `dev` as an
ordinary commit; A1 sweeps it into the next real release. Say that is what
happened.

### A4. Gate it

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo build -p orbs
```

### A5. Commit and push to `dev`

1. Stage **specific paths only** — never `git add -A`, never `git add .`:
   `docs/CHANGELOG.md`, plus `Cargo.toml Cargo.lock` if A2 opened a version,
   plus the paths the user actually changed. Enumerating forty of them is fine;
   sweeping is not, because a stray deletion or a local-only file rides along
   invisibly.
2. Commit in the format above.
3. `git push origin dev` (`-u` the first time).

Then show the block, the rendered Bluesky post, the gate result, and the run
`dev-release.yml` started. Say which version this landed in and whether it
opened a new one.

Do not touch `main`.

---

## Mode C — Consolidate (`consolidate` argument)

Many dev pushes leave the open block bloated: near-duplicates, several bullets
about one area, and bullets that **cancel out** — a bug introduced in one push
and fixed in a later one, a feature added then reworked. Players who only ever
see `main` never experienced the in-between states, so the block should describe
only the **net** change since the last release.

### C1. Preconditions

1. On `dev`, working tree clean. If dirty, stop — run dev mode first.
2. `git fetch --tags`, and confirm the top block is an untagged `## [vV]`
   matching `Cargo.toml`. If its tag exists, that version shipped; stop.

### C2. Ground truth

`git diff main dev --stat`, and `git diff main dev` for detail, is the actual net
change since the last promotion. Every surviving bullet must correspond to
something in it.

### C3. Rewrite

- **Merge** bullets about the same area into one describing its final state.
- **Supersede** — when a later bullet reworks an earlier one, keep only the
  result.
- **Cancel** — if a change and its reversal both happened inside this version,
  drop both.
- **Keep** distinct, still-true changes.
- Re-sort into `### Added` / `### Changed` / `### Fixed`; drop empty
  subsections; redo `### Description`.

Keep the heading, refresh the date. Consolidate never changes the version.

If everything cancels, leave the heading bare and say so — promoting an empty
block is almost certainly not what the user wants.

### C4. Commit and push to `dev`

Same as A5, staging `docs/CHANGELOG.md` only.

---

## Mode B — Promotion (`main` argument)

A pure fast-forward. **It edits no files, creates no commit, and bumps nothing.**
If you want to change a file here, the flow has gone wrong — fold it into a dev
release instead.

### B1. Preconditions — every one a hard stop

1. `git rev-parse --abbrev-ref HEAD` is `dev`.
2. `git status` is clean.
3. `git fetch --tags`.
4. Read `V` from `Cargo.toml`. Tag `vV` must **not** exist.
5. The top block of `docs/CHANGELOG.md` is `## [vV]` matching `V`. If it names a
   different version, run dev mode to repair it. **If there is no block at all,
   stop** — promoting would push a commit that releases nothing, silently.
6. `git rev-parse dev origin/dev` must match. If local `dev` is ahead, push it
   first.
7. **The dev tip must have a green `dev-release.yml` run:**
   ```bash
   gh run list --workflow=dev-release.yml --commit "$(git rev-parse HEAD)" \
     --json databaseId,status,conclusion,url
   ```
   - **empty** → stop; nothing has gated this commit.
   - **failure/cancelled** → stop and surface it.
   - **in_progress** → allowed, but say so: `release.yml` will re-run the gate
     anyway, so a failure surfaces after `main` has already moved.
   - **success** → ideal.

### B2. Fast-forward and push

1. `git switch main`
2. `git pull --ff-only`
3. `git merge --ff-only dev` — if this fails, stop and surface it. Never
   force-push, never rebase silently.
4. `git rev-parse main dev` to confirm the SHAs match. If they do not, stop —
   something other than a fast-forward happened.
5. `git push origin main`. **This is the release**: it triggers `release.yml`,
   which tags `vV`, publishes the GitHub Release, and posts to Discord and
   Bluesky. B1 is what stands between the argument and this line; by the time you
   reach it, the decision was made when the user typed `main`.
6. `git switch dev` — always, even if the push failed. Leaving the user on
   `main` means their next commit lands there.

### B3. Report

- the version now on `main`, and that promotion changed no files;
- that `release.yml` is running — gate, tag, GitHub Release, Discord, Bluesky —
  with the URL (`gh run list --workflow=release.yml --limit 1`);
- **do not call it live until that run is green.** The tag is pushed several
  minutes before the announcements go out.

If a step fails after the tag exists, the retry is *Actions → Release → Run
workflow* with **`force: true`** — the tag-exists guard would otherwise skip
everything. It re-announces.

*Missing until Steam returns:* no BBCode patch notes to paste into Steamworks, no
build to set live, and no phone prompt to approve. When those come back,
promotion stops being the end of the story and announcing moves out of
`release.yml` — see the commented block at the bottom of that file.

---

## Hard rules (all modes)

- **Invoking the skill approves the push its mode implies**, including `main`.
  Ask when a precondition fails, never to re-confirm the argument.
- **Never `git add -A` or `git add .`** — stage only what the user changed.
- **Never `git reset`**, and never `git checkout` to discard working-tree
  changes. `git switch` for branches only.
- **Never push to `main` directly.** Promotion fast-forwards `dev` only.
- **Never force-push.** If a fast-forward is impossible, stop and ask.
- **Never any AI or tool attribution in a commit message.**
- **Never edit a file in promotion mode.**
