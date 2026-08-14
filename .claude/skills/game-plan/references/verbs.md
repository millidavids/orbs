# New verb

The commonest addition, and the one with the most places to forget. A verb is
not a function — it is an entry in a parser table, a set of synonyms in three
registers, a scope, a handler, a manual page, and outcome prose. Miss one and it
fails somewhere specific and quiet.

Worked example to read in full before writing: **`research`** — `git log -S
'Research' --oneline` finds the commit that added it, and it touches every site
below.

## Integration points

Verified by `grep -rn "Verb::Research" crates/orbs-sim/src`, which is the check
to re-run when this list looks stale.

| File | What goes in it |
|---|---|
| `parser/verb.rs` | the variant; `Verb::ALL` (**bump the array length**); `canonical`, `participle`, `signature`, `signature_label`, `group`; the `is_operation` / `transmutes` / `is_destructive` arms |
| `parser/vocabulary.rs` | `SYNONYMS` — the arcane, plain and shell registers. §6 wants all three reachable |
| `execute/dispatch.rs` | the match arm to the handler, and `is_live` / `is_gated` if it is not yet built or is progression-locked |
| `execute/<feature>.rs` | the body. A new file if it is a new concern; CLAUDE.md is feature-sliced |
| `tower/scene.rs` | `Scene::offering(verb)` **if it belongs to an instrument** — §7 scopes an instrument's verb to where the instrument is |
| `tower/build.rs` | any nodes the verb needs raised in the world |
| `content/prose.toml` | `man_<verb>_{use,gloss,1..n,eg1..n,also}` **plus** every outcome line the handler says |

`tower/panel.rs` and `tower/spell/block.rs` come in too if the verb drives an
instrument or is usable inside a spell.

## `Verb::ALL` is a fixed-length array

`pub const ALL: [Self; N]`. Adding a variant without bumping `N` is a compile
error, which is the point — but the *tests* that count offered verbs will also
move, and they are asserting a real property. Read what they claim before
changing a number.

## Three lints will fail until the prose exists

They are in `execute/recall.rs`'s tests and they are not optional:

- **`every_verb_has_a_page`** — `_use`, `_gloss` and at least one `_1` for every
  member of `Verb::ALL`.
- **`a_synopsis_names_every_slot_its_signature_requires`** — the authored
  `_use` line must open with the canonical and name each required slot's label.
  The synopsis is **authored, not generated**: `signature()` carries no
  connectives, so a generated `move` would read `move reagent place place`.
- **`a_page_never_claims_a_word_that_is_not_there`** — every `see also` must
  resolve. A manual pointing at a word the parser lacks is worse than one
  pointing nowhere, because the player types it.

A verb that is not live or is gated **still gets a page**, saying so. `undo`'s
page admits it is not built; `bind`'s says what it costs. §15 wants the dead-end
rate low, and a page that admits a word does nothing is the cheapest way there.

## Choosing the noun kind

`NounKind` is the surface a slot resolves against, and the wrong one leaks.
`NounKind::Any` reaches `Topic`, so registering something there puts it into tab
completion, into `compile::fix`'s spell-condition resolution, and into the
bare-argument numbered prompt. That is why verb canonicals are `Command` — a
kind `Any` refuses — reachable only from the `Subject` slot kind. §19 records the
three leaks in full.

If a new kind is needed, decide what must *not* reach it before deciding what
must.

## Fuzzy matching has a brake

A phrase that is itself a known substance only ever matches **exactly**
(`Scene::knowing`). Typos still fuzz; a real name is never read as a different
real name. If a verb introduces a new class of nameable thing, ask whether it
belongs in that vocabulary — `digest ground-sage` once digested ground-salt.

## Scoping

Two kinds of verb:

- **Core** — `attend`, `survey`, `peruse`. Available everywhere, because they
  are how you *reach* a domain and gating them locks the key inside the door.
- **Operation** — `grind`, `digest`, `research`. `is_operation()` is true, and
  `Scene::offering` makes it a word only where its instrument stands. Out of its
  domain the reading never exists, so it cannot win a tie or capture a typo
  meant for another domain's verb. **This is what keeps the vocabulary from
  growing without bound** as §10's five further domains land.

An operation typed elsewhere resolves to `Resolution::Elsewhere` — *"not here"*,
never *"I do not know that word"*, which would lie about a word the game taught
the player in the room next door.

## Records, not sentences

The handler emits records; prose is looked up by key and attached. If the screen
already shows the fact — the map has just moved — use `RecordBuilder::quiet`, so
it is logged without being drawn. **Check the record is actually filed**: a
domain's log matches `Source`, `Path` or `Origin` against the domain and what
stands in it, and a quiet record no log holds is not logged at all.

## See it

```bash
ORBS_BOOT=0 ORBS_DUMP="attend <domain>; <verb> <argument>" cargo run -p orbs
ORBS_BOOT=0 ORBS_DUMP="attend <domain>; help" cargo run -p orbs      # it lists
ORBS_BOOT=0 ORBS_DUMP="recall <verb>" cargo run -p orbs              # its page
ORBS_BOOT=0 ORBS_DUMP="attend <elsewhere>; <verb>" cargo run -p orbs # refuses in voice
```

Check `help` at the floor too — `ORBS_GRID=80x22` — because a 25-verb listing is
most of the transcript there and that is where it has to read.
