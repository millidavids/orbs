# O.R.B.S. — Roadmap

**Status tracker. Derived from [DESIGN.md](DESIGN.md) §15, which is authoritative.**
If the two disagree, DESIGN.md wins and this file is wrong.

Last updated: 2026-08-03 · **Phase 0 not yet started**

---

## Shape

Solo, commercial, Steam. **28 months of work against a 24-month target** — that
gap is deliberate and is the cut line's job to close, not a scheduling error to
hide. Release posture: demo first, then full 1.0. No Early Access.

| Phase | Months | Words | Status |
|---|---|---|---|
| 0. Vertical slice | 4 | ~3k | ⬜ Not started |
| 1. Core loop | 5 | ~15k | ⬜ |
| 2. Siege | 4 | ~15k | ⬜ |
| 3a. Breadth | 4 | ~18k | ⬜ |
| 3b. Remote hosts | 3 | ~12k | ⬜ |
| 3c. Engine upgrade | 1 | — | ⬜ |
| 4. Onboarding + demo | 4 | ~20k | ⬜ |
| 5. Ship | 3 | ~5k | ⬜ |

---

## Phase 0 — Vertical slice

**Exit gate is numeric and was set before any results exist:**

> ≥ 8 external testers, at least half self-reporting no shell experience, complete
> a 15-minute scripted scenario with expected-intent ground truth.
> - **≥ 85% of inputs resolve to the intended action on first attempt** (below 70%
>   is a no-go), **and**
> - **≥ 95% of initially-unresolved inputs reach the intended action within two
>   further attempts, with zero dead ends.**
>
> Act on the per-input failure *clustering*, not the aggregate — 8 testers over 15
> minutes has wide confidence intervals, so 84% vs 86% is noise.

### Work items

- [ ] **Workspace + determinism spine** — five crates; `orbs-sim` with zero Bevy
      dependency; seeded per-subsystem RNG; single `step(world, tick)` entry point
- [ ] **`orbs-render` Frame boundary** — cell buffer, semantic styling, layout.
      Cannot be retrofitted
- [ ] **Parser + instrumentation** — the 16 slice commands (DESIGN.md §6.1) across
      three registers; full input/resolution/candidate-score capture with export
- [ ] **Naming pass** for the slice's 16 commands — *done*, DESIGN.md §6.1
- [ ] **Cell-grid text renderer** — glyph atlas + instanced quads, integer
      fidelity tiers
- [ ] **CRT port** — 2,638 lines + 242-line shader from `court_wizard`, Bevy
      0.18.1 → 0.19, made cell-size-aware
- [ ] **Structured-record output model** — linear, semantic, presentation separate
- [ ] **Brewing + archive** — the two starting domains, both thin
- [ ] **Log-poisoning sabotage** on brewing logs, via `peruse` / `sift` / `verify`
- [ ] **Boot sequence** — status report reflecting real world state, sticky skip
- [ ] **Scaffold tutorial** — throwaway, exists so the gate measures the parser
      rather than the absence of onboarding
- [ ] **Worst-case legibility test** — tier 2 at minimum supported window, four
      panes, siege in progress, peak-threat CRT, eldritch active, tester must spot
      a single-character sabotage tell. Also establishes the minimum window at
      which tier 2 is offered
- [ ] **Dev ergonomics** — `bevy/dynamic_linking`, fast linker (lld/mold)
- [ ] **Run the gate**

---

## Phase 1 — Core loop

**Exit:** a player automates a duty and feels clever; non-terminal testers are in
the loop.

- [ ] World-clock implementation per DESIGN.md §5.0
- [ ] Script engine — bind-time canonicalisation, ID-anchored referents, execution
      budget, failure taxonomy, Attention pool
- [ ] Remaining sabotage surfaces (world, script text, trigger clocks)
- [ ] Third domain (scrying — the player's first discovery)
- [ ] Minimal apprenticeship + **continuous non-terminal-user playtesting**
- [ ] Content data format — hot-reloadable TOML/RON, prose never in Rust literals
- [ ] `orbs-balance` CLI sweeping §11.5's first-pass numbers
- [ ] Scrappy internal `orbs-tui` as a dev tool (no parity/polish obligation)
- [ ] Trace tuning — accrual rates, composition rule, ceiling, threshold
- [ ] Naming pass for the remaining ~35 canonical commands (≤7 chars, §6.1 rule)
- [ ] Hidden-directory authoring plan (~80 fragments' worth)

---

## Phase 2 — Siege

**Exit:** sieges are tense and scripts visibly matter.

- [ ] Autobattler
- [ ] Procedural trait composition
- [ ] Adversarial aberrations across all four surfaces
- [ ] Escrow economy (progress-scaled, 20% floor, 50% completion bonus)
- [ ] Unattended-siege backlog + dispersal + decay
- [ ] Pane addressing and Focus-slot reservation
- [ ] One siege type, end to end
- [ ] Drift stub
- [ ] 21-pair synergy template (shared mechanics, bespoke flavour line)
- [ ] Siege type definitions + completion-fraction formula for each
- [ ] Difficulty tiers within the progression-gated range

---

## Phase 3a — Breadth

- [ ] All 7 domains
- [ ] Discovery / research loop
- [ ] Full drift
- [ ] Offline progression + its unlock
- [ ] Shared-engine extraction (deferred here deliberately — the generalisation
      axis is only knowable from the second consumer)

## Phase 3b — Remote hosts

- [ ] Host filesystems, verbs, infiltration
- [ ] Trace amplification
- [ ] Ship-quality `orbs-tui` *if the schedule allows* — **cut-line item 3**

## Phase 3c — Engine upgrade

- [ ] Bevy version window, isolated from new-system work
- [ ] Green on all three platforms

---

## Phase 4 — Onboarding + demo

- [ ] Diegetic apprenticeship, polished
- [ ] Progressive reveal throughout
- [ ] In-world grimoire (`help` / `man`)
- [ ] Soft ending
- [ ] **Demo + capsule + trailer** — the trailer leads on CRT *motion*, which a
      static capsule cannot carry
- [ ] Wishlist target set; revisit the $4 price against actual content volume

## Phase 5 — Ship

- [ ] Accessibility pass (see DESIGN.md §14)
- [ ] Screen-reader siege mode (ticks advance on player input)
- [ ] Options, remapping, all toggles
- [ ] Steam integration and depot
- [ ] Polish

---

## Cut line — decided in advance

Drop in this order under schedule pressure:

1. Steam Workshop (already deferred)
2. OS-window pane detaching (already deferred)
3. Ship-quality `orbs-tui` (boundary and dev-tool build kept regardless)
4. Remote hosts → read-only scrying feeds
5. Enchanting, then Summoning (extraction task cut with them)
6. Screen-reader support slips further post-launch (architecture retained)
7. Multiplexing capped at two panes

**Never cut:** the parser, the script engine, the aberration system, one excellent
siege, onboarding.

---

## Live open questions

Tracked in DESIGN.md §18. **Nothing currently blocks Phase 0.**
