//! Bringing an older document up to [`FORMAT`]. What each bump changed is
//! listed on [`FORMAT`]; what each migration does is [`migrate`].

use super::document::{FORMAT, GarrisonSave, Save};

/// Bring an older document up to [`FORMAT`], or say why it cannot be.
///
/// A save is refused only when the new model cannot state what the old one
/// meant — the ward rework changed what a *scored reading* means — not merely
/// because the number went up. Keep the table level with the function: two rows
/// were once missing for migrations `migrate` had been performing since
/// `0.8.13`.
///
/// | Bump | What changed | Migratable |
/// |---|---|---|
/// | 1 → 2 | The lens's scoring model | No — old readings mean nothing now |
/// | 2 → 3 | `RngStream::COUNT` 8 → 9, and the `battlements/` → `sanctum/` rename in the same commit | No — a format-2 save names a room and four fixtures this build lacks, and `adopt` addresses nodes by path |
/// | 3 → 4 | `RngStream::COUNT` 9 → 10 (`Menagerie`) | Yes — pad |
/// | 4 → 5 | `bide until` withdrawn from the language | Yes, with the caveat below |
/// | 5 → 6 | `RngStream::COUNT` 10 → 11 (`Siege`) | Yes — pad |
/// | 6 → 7 | `/tower/bailey/host` → `/tower/bailey/enemy`, the first *content* rename | Yes — rewrite the path |
/// | 7 → 8 | `Siege::quintessence`, a resource a running siege was fought without | Yes — fill the pool |
/// | 8 → 9 | `RngStream::COUNT` 11 → 12 (`Forge`), and quintessence moving from the siege to the tower | Yes — pad, and lift the pool |
/// | 9 → 10 | `[progress]` gains `tally`, `reached` and `opened` | Yes — every field defaults to the honest reading; `opened` absent is *everything open* |
/// | 10 → 11 | `[world]` gains `length`, `[progress]` gains `stores` | Yes — `length` absent is the curve as authored, `stores` absent is *full* |
/// | 11 → 12 | The menagerie's chant replaced by the circle: four syllable nodes and their readings, gone | Yes — drop the nodes; a figure mid-song loads as a circle with no beast |
/// | 12 → 13 | `[progress] opened` gains `circle`, whose absence draws lesser beasts | Yes — an open tower gains the key; a sealed one catches up on its own |
/// | 13 → 14 | A waiting beast gains `turned`, its turned wires | Yes — absent is none turned, which is what it was |
/// | 14 → 15 | Calm-layer log drift removed | Yes — clear every poisoned log |
/// | 15 → 16 | A siege gains `fortified` and the `Stilled` outcome; stillness distils from `stilling-draught` | Yes — absent is nought, and a quiet-draught in the alembic is renamed |
/// | 16 → 17 | Troops leave the arsenal for a standing garrison | Yes — held troops become bodies, two each, up to the most; to the reserve beside the levied line if a siege is open |
/// | 17 → 18 | `[progress]` gains open supplies, a siege its freshness at `defend`; vigour's room stops being for the whole siege | Yes — absent is none open, a siege reads freshness now, and a room already bought is kept |
/// | 18 → 19 | `[progress] opened` gains `demon:<name>`, whose absence binds nothing | Yes — an open tower gains every demon; a sealed one catches up on its own |
///
/// Padding a stream is exact: `Rngs::restore` winds a stream forward from the
/// master seed by the stored position, so nought is precisely a world that
/// never drew from it.
///
/// The language bump's caveat is survivable. A saved `bide until` compiles to a
/// line the orb cannot read — a fault the player can see and fix, in a file
/// whose text is theirs (§19) — and §8's taxonomy is *"scripts always log and
/// never halt"*.
pub(super) fn migrate(mut save: Save) -> Result<Save, super::SaveError> {
    /// The oldest format whose meaning survives into the current model.
    ///
    /// 3, not 2: the 2 → 3 bump shipped with the `battlements/` → `sanctum/`
    /// rename (`de29565`, v0.4.2), and `adopt::apply` addresses nodes by path,
    /// so accepting a format-2 document dropped the entire sanctum without a
    /// word. Not a string rewrite like `host` → `enemy` either — the old
    /// `rampart` is the *bailey's* fixture name now, so a format-2 path would
    /// have to be disambiguated by its parent.
    const OLDEST: u32 = 3;

    if save.world.format < OLDEST {
        return Err(super::SaveError::Behind {
            found: save.world.format,
            understood: FORMAT,
        });
    }

    // Pad, never truncate: a document with more streams is `Ahead` and was
    // refused above, so anything here is short or exact.
    while save.rng.positions.len() < crate::RngStream::COUNT {
        save.rng.positions.push("0".to_owned());
    }

    // The bailey's second band was `host` until format 7; a node is addressed
    // by path, so the rename is a string rewrite and nothing more.
    if save.world.format < 7 {
        const WAS: &str = "/tower/bailey/host";
        const NOW: &str = "/tower/bailey/enemy";
        for node in &mut save.nodes {
            if node.path == WAS {
                node.path = NOW.to_owned();
            }
        }
    }

    // A siege fought before quintessence existed opens with a full pool: nought
    // is not the fight that was saved, and the world it would be scaled against
    // is not in the document. Generous is the honest way to be wrong, and it
    // lasts one siege.
    if save.world.format < 8 {
        for node in &mut save.nodes {
            if let Some(siege) = node.siege.as_mut()
                && siege.quintessence.is_none_or(|held| held == 0)
            {
                siege.quintessence = Some(crate::tower::QUINTESSENCE_BASE);
            }
        }
    }

    // 8 → 9: the pool came up to the tower, the forge spending what a siege
    // used to spend privately (§19). The largest is lifted — exact in practice,
    // there being one bailey. A save with no siege gets the ceiling rather than
    // nought, which would read as an inert forge; the ceiling is not in the
    // document, so `None` travels and `restore` fills it.
    if save.world.format < 9 {
        let carried = save
            .nodes
            .iter_mut()
            .filter_map(|node| node.siege.as_mut())
            .filter_map(|siege| siege.quintessence.take())
            .max();
        save.progress.quintessence = carried;
    }

    // 11 → 12: the chant's nodes go, by prefix rather than path — a syllable's
    // readings are children and an orphan would be restored at the root. The
    // circle's children go too: a leftover `remaining` makes `if the circle is
    // empty` false for ever, and they are republished on the next `summon`.
    if save.world.format < 12 {
        const ROOM: &str = "/tower/menagerie/";
        const SYLLABLES: [&str; 4] = ["skyward", "earthward", "leftward", "rightward"];
        let chant = |path: &str| {
            let Some(rest) = path.strip_prefix(ROOM) else {
                return false;
            };
            let head = rest.split('/').next().unwrap_or(rest);
            SYLLABLES.contains(&head) || rest.starts_with("circle/")
        };
        save.nodes.retain(|node| !chant(&node.path));
    }

    // 12 → 13: an open tower gains the key it could not have named. A sealed
    // one is left to `mastery::caught_up`.
    if save.world.format < 13
        && !save.world.sealed
        && let Some(opened) = save.progress.opened.as_mut()
        && !opened.iter().any(|key| key == crate::tower::opened::CIRCLE)
    {
        opened.push(crate::tower::opened::CIRCLE.to_owned());
    }

    // 14 → 15: before this no round could reach a log, so every lying one is
    // the calm layer's, siege or no siege (§19).
    if save.world.format < 15 {
        for node in save.nodes.iter_mut().filter(|node| node.log) {
            node.poisoned = false;
        }
    }

    // 15 → 16: the alembic distils stillness from `stilling-draught` now, and a
    // run is matched again when it lands. A quiet-draught already in the
    // alembic would land on no recipe and foul, so it becomes the draught the
    // new recipe wants. Generous, once: that brew skips the scroll.
    if save.world.format < 16 {
        const WAS: &str = "/alembic/quiet-draught";
        const NOW: &str = "/alembic/stilling-draught";
        for node in &mut save.nodes {
            if let Some(room) = node.path.strip_suffix(WAS) {
                node.path = format!("{room}{NOW}");
            }
        }
    }

    // 16 → 17: troops left the arsenal (§19, *Supply*). Each was two bodies on
    // the wall, so that is what it brings home, up to the floor's most. Mid-siege
    // the line is already levied, so they wait in the reserve beside it, as
    // `enlist` would leave room. The Ley grant is not in the document and is not
    // counted; the cap only errs low.
    if save.world.format < 17 {
        use crate::tower::siege::{GARRISON_FLOOR, GARRISON_MOST};
        const TROOP: &str = "troop";
        let held: u32 = save
            .nodes
            .iter()
            .filter(|node| node.path.rsplit('/').next() == Some(TROOP))
            .filter_map(|node| node.stock.as_deref()?.parse::<u32>().ok())
            .sum();
        save.nodes
            .retain(|node| !node.path.split('/').any(|segment| segment == TROOP));
        let bodies = held.saturating_mul(2);
        let levied = save
            .nodes
            .iter()
            .filter_map(|node| node.siege.as_ref())
            .find(|siege| siege.running())
            .map(|siege| siege.levied);
        if save.progress.garrison.is_none() {
            save.progress.garrison = Some(if let Some(levied) = levied {
                GarrisonSave {
                    standing: 0,
                    reserve: bodies.min(GARRISON_MOST.saturating_sub(levied)),
                }
            } else {
                GarrisonSave {
                    standing: GARRISON_FLOOR.saturating_add(bodies).min(GARRISON_MOST),
                    reserve: 0,
                }
            });
        }
    }

    // 17 → 18: vigour's room was bought for the rest of the siege, and a
    // fortify supply now holds its room a round at a time. A siege saved
    // mid-fight keeps what it paid for (§19, *Supply*).
    if save.world.format < 18 {
        for siege in save.nodes.iter_mut().filter_map(|node| node.siege.as_mut()) {
            siege.kept = siege.fortified;
        }
    }

    // 18 → 19: an open tower gains the demons it could not have named. A sealed
    // one is left to `mastery::caught_up`, as 12 → 13 left the circle.
    if save.world.format < 19
        && !save.world.sealed
        && let Some(opened) = save.progress.opened.as_mut()
    {
        for demon in crate::content::Spendables::builtin().demons() {
            let key = crate::tower::opened::demon_key(demon);
            if !opened.contains(&key) {
                opened.push(key);
            }
        }
    }

    save.world.format = FORMAT;
    Ok(save)
}
