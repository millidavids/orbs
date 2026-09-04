//! What the tower has opened: rooms, recipes, charms, and the wall (DESIGN.md
//! §11.5, §19).
//!
//! # One set, four kinds of key
//!
//! `domain:archive`, `recipe:warding`, `charm:whetted`, `siege`. A station on
//! either track may `opens` any of them, and every question of the form *"may
//! the player do this yet"* is answered here — `attend` for a room, a recipe
//! firing for a product, `imbue` for a charm, `defend` for the wall. One holder
//! rather than one per kind, so a save carries one list and a restore cannot
//! open a room and forget a recipe.
//!
//! **Not [`Learned`]**, which stays the lens's. A secret recipe
//! is *found* — rolled on a broken ward — and a gated one is *earned* by doing
//! the work of the room it belongs to. They are two sources of the same
//! capability, and `learn` filters against `secrets()` on purpose, so a gated
//! name routed through it would open nothing and say nothing.
//!
//! # A start state, not a world rule
//!
//! [`Opened::all`] is the tower every test, dump, balance policy and `screens`
//! example has always used: everything open. `Sim::new` builds that. A fresh
//! *game* starts sealed — the laboratory and whatever no station opens — and
//! earns the rest, which is `Sim::sealed`'s tower. §19 records the trade-off:
//! ~200 See-it lines and every `tests/*.rs` keep meaning what they meant, at the
//! cost of the dump's default differing from a fresh game by construction, the
//! same class of difference `ORBS_BOOT=0` already is.
//!
//! # A sealed room is sealed everywhere it is a noun, through one gate
//!
//! `attend` resolves any place tower-wide, so a room-name check alone is
//! bypassed by `attend stacks`; `survey <place>`, the boot report, the scene
//! (and so completion, `recall` and `peruse`) and calm-layer sabotage all reach
//! a room too. [`sealed_room_of`] is the one question, and [`Sealed`] is the
//! same answer as a marker on every node under a sealed room, for the queries
//! that cannot ask a function.

use std::collections::BTreeSet;
use std::fmt;

use bevy_ecs::prelude::*;
use orbs_render::{FieldName, RecordKind, Role};

use crate::content::{Charms, Progression, Prose, Recipes};
use crate::session::Scrollback;

use super::Learned;
use super::node::{Name, children_of};

/// The key that arms the wall.
pub const SIEGE: &str = "siege";

/// One thing that can be opened, parsed from its authored key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    /// A room: `domain:<name>`.
    Domain(String),
    /// A gated product: `recipe:<name>`.
    Recipe(String),
    /// A charm the forge may lay: `charm:<kind>`.
    Charm(String),
    /// The wall: `siege`.
    Siege,
}

impl Key {
    /// Read an authored key.
    ///
    /// `None` for a key of no kind, which `Progression::check` turns into a load
    /// failure — a key nothing reads would open nothing and look authored.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        if text == SIEGE {
            return Some(Self::Siege);
        }
        let (kind, name) = text.split_once(':')?;
        if name.is_empty() {
            return None;
        }
        match kind {
            "domain" => Some(Self::Domain(name.to_owned())),
            "recipe" => Some(Self::Recipe(name.to_owned())),
            "charm" => Some(Self::Charm(name.to_owned())),
            _ => None,
        }
    }

    /// The noun, for a sentence about what opened.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Domain(name) | Self::Recipe(name) | Self::Charm(name) => name,
            Self::Siege => SIEGE,
        }
    }

    /// Which prose line says it opened: `opened_<kind>`.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Domain(_) => "domain",
            Self::Recipe(_) => "recipe",
            Self::Charm(_) => "charm",
            Self::Siege => SIEGE,
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Siege => f.write_str(SIEGE),
            other => write!(f, "{}:{}", other.kind(), other.name()),
        }
    }
}

/// Every place that can be shut.
///
/// **Wider than [`DOMAINS`](super::DOMAINS), and the difference is two rooms.**
/// That list is the rooms you *work in* — the ones with a mastery line and a
/// rail box. Sealing asks a different question: the **bailey** is a room you
/// fight in and the **grimoire** is where your spells live, and both can be shut
/// before they are earned without either being somewhere you tend.
///
/// **A list as well as a predicate**, because a caller that wants to *walk* the
/// shut rooms had nowhere to get them and reached for `DOMAINS` instead — which
/// is how `tower::scene` stopped withholding a shut grimoire's name the moment
/// the grimoire left that list. One list and one predicate over it is the whole
/// rule; §19 records more defects from two expressions of one rule than from
/// anything else, and this rule has had four sites.
pub const ROOMS: [&str; super::DOMAINS.len() + 2] = {
    let mut rooms = [""; super::DOMAINS.len() + 2];
    let mut index = 0;
    while index < super::DOMAINS.len() {
        rooms[index] = super::DOMAINS[index];
        index += 1;
    }
    rooms[index] = super::siege::BAILEY;
    rooms[index + 1] = super::GRIMOIRE;
    rooms
};

/// Whether `name` is a place that can be shut. See [`ROOMS`].
#[must_use]
pub fn is_room(name: &str) -> bool {
    ROOMS.contains(&name)
}

/// The key for a room.
#[must_use]
pub fn domain_key(name: &str) -> String {
    format!("domain:{name}")
}

/// The key for a gated product.
#[must_use]
pub fn recipe_key(name: &str) -> String {
    format!("recipe:{name}")
}

/// The key for a charm.
#[must_use]
pub fn charm_key(kind: &str) -> String {
    format!("charm:{kind}")
}

/// Everything the tower has opened, by key.
///
/// **A `BTreeSet`, so a save's bytes do not depend on the order things were
/// opened in** — the argument `Learned::known` makes for its own set.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct Opened(BTreeSet<String>);

impl Opened {
    /// Every room, every gated product, every charm, and the wall.
    ///
    /// The tower `Sim::new` builds, and the one a save from before sealing
    /// existed restores to — see the module header.
    #[must_use]
    pub fn all(recipes: &Recipes, charms: &Charms) -> Self {
        let mut keys: BTreeSet<String> =
            super::DOMAINS.iter().map(|name| domain_key(name)).collect();
        // The two rooms that are not in `DOMAINS` and can still be shut — see
        // `is_room`, which is the one place that rule lives.
        keys.insert(domain_key(super::siege::BAILEY));
        keys.insert(domain_key(super::GRIMOIRE));
        keys.extend(recipes.gated().into_iter().map(recipe_key));
        keys.extend(charms.names().map(charm_key));
        keys.insert(SIEGE.to_owned());
        Self(keys)
    }

    /// What a fresh game starts with: everything no station opens.
    ///
    /// **Derived from the content, not authored twice.** The laboratory is open
    /// because nothing opens it; `hurried` is open because no station names it;
    /// the archive is shut because `laboratory_1` opens it. A second list of
    /// starting keys would be a second expression of the same rule, which is the
    /// defect §19 records most. The bailey follows the wall: it is shut while
    /// `siege` is something a station opens.
    #[must_use]
    pub fn start(recipes: &Recipes, charms: &Charms, curve: &Progression) -> Self {
        let mut keys = Self::all(recipes, charms).0;
        let opened_by_a_station: BTreeSet<&str> = curve
            .ley_line()
            .iter()
            .flat_map(|station| station.opens.iter())
            .chain(
                curve
                    .mastery()
                    .iter()
                    .flat_map(|milestone| milestone.opens.iter()),
            )
            .map(String::as_str)
            .collect();
        for key in &opened_by_a_station {
            keys.remove(*key);
        }
        if opened_by_a_station.contains(SIEGE) {
            keys.remove(&domain_key(super::siege::BAILEY));
        }
        Self(keys)
    }

    /// Whether `key` has been opened.
    #[must_use]
    pub fn has(&self, key: &str) -> bool {
        self.0.contains(key)
    }

    /// Whether the room named may be entered.
    #[must_use]
    pub fn is_open(&self, domain: &str) -> bool {
        self.has(&domain_key(domain))
    }

    /// Every key, in the set's order.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(String::as_str)
    }

    /// Put a set back, for a save.
    pub(crate) fn restore(&mut self, keys: impl IntoIterator<Item = String>) {
        self.0 = keys.into_iter().collect();
    }

    /// Open `key`. Whether that changed anything.
    fn insert(&mut self, key: &str) -> bool {
        self.0.insert(key.to_owned())
    }
}

/// Whether this tower began sealed.
///
/// **Part of the recorded start**, beside the seed: a sealed and an open tower
/// with identical seed and submissions diverge at the first `attend archive`,
/// so a replay has to know which it is rebuilding. Saved in `[world]`.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Sealing(pub bool);

/// A node under a room the player may not enter yet.
///
/// **The same fact as [`Opened`], as a marker**, for the systems that query
/// rather than ask: calm-layer sabotage picks its target from a `Query`, and a
/// query cannot walk to a node's room and consult a resource. [`seal`] keeps
/// the markers in step with the set, and is the only writer of them.
#[derive(Component, Debug, Clone, Copy)]
pub struct Sealed;

/// Open `key`, and say whether it was shut before.
///
/// **Returns whether the state changed**, and the caller says the sentence
/// only then: a tower that restored open and re-reaches the station that opens
/// the archive must not announce an archive it already has.
///
/// The wall opens the bailey with it — a room to defend from is what `siege`
/// arms — and a room opening clears its markers.
pub fn open(world: &mut World, key: &str) -> bool {
    let changed = world.resource_mut::<Opened>().insert(key);
    if key == SIEGE {
        world
            .resource_mut::<Opened>()
            .insert(&domain_key(super::siege::BAILEY));
    }
    if changed && (key == SIEGE || key.starts_with("domain:")) {
        seal(world);
    }
    changed
}

/// Open every key a station carries, and say each one that was shut.
///
/// **Both tracks open things, and this is the one loop that does it.** A step
/// on the Ley Line opens the grimoire and the forge; a mastery station opens
/// everything else. Two copies of this would be two places to forget the rule
/// that only a *change* is announced — which is the rule a migrated tower
/// depends on, since it re-reaches stations whose rooms it already stands in.
pub(crate) fn opening(world: &mut World, keys: &[String]) {
    for key in keys {
        if !open(world, key) {
            continue;
        }
        let Some(parsed) = Key::parse(key) else {
            continue;
        };
        let message = world.resource::<Prose>().line(
            &format!("opened_{}", parsed.kind()),
            &[("name", parsed.name())],
        );
        world
            .resource_mut::<Scrollback>()
            .records_mut()
            .push(RecordKind::Completion)
            .text(FieldName::Name, parsed.name())
            .text(FieldName::Kind, parsed.kind())
            .text(FieldName::Message, &message)
            .role(Role::Success)
            .finish();
    }
}

/// Bring every room's [`Sealed`] markers into step with [`Opened`].
///
/// Idempotent, and called wherever the set changes wholesale: at a sealed
/// construction, after a restore, and when a room opens. A marker on every
/// node under a shut room, none under an open one.
pub fn seal(world: &mut World) {
    let root = super::root(world);
    let rooms: Vec<(Entity, bool)> = children_of(world, root)
        .into_iter()
        .chain(
            children_of(world, root)
                .into_iter()
                .flat_map(|child| children_of(world, child)),
        )
        .filter_map(|node| {
            let name = world.get::<Name>(node)?.0.clone();
            is_room(&name).then(|| (node, world.resource::<Opened>().is_open(&name)))
        })
        .collect();
    for (room, open) in rooms {
        let mut pending = vec![room];
        while let Some(node) = pending.pop() {
            pending.extend(children_of(world, node));
            if open {
                world.entity_mut(node).remove::<Sealed>();
            } else {
                world.entity_mut(node).insert(Sealed);
            }
        }
    }
}

/// The shut room `node` stands in, if it stands in one.
///
/// **The one gate.** `attend`, `survey`, the scene and the boot report all ask
/// this rather than checking a name against the set, because a node reached by
/// path — `attend stacks` — has a room the name alone would not mention.
///
/// **It walks the ancestors itself rather than asking `domain_of`.** A domain is
/// a child of `/tower` and the grimoire is not — it is `/tower`'s *sibling* at
/// `/grimoire`, which `domain_of` documents as one of the two nodes it answers
/// `None` for. So every node inside a shut grimoire answered *"not in a shut
/// room"* while [`seal`] — which walks the tree — had marked all twenty of them.
/// Two representations of one fact, disagreeing, which is exactly what the
/// module header says this pair exists to prevent.
#[must_use]
pub fn sealed_room_of(world: &World, node: Entity) -> Option<String> {
    let mut at = node;
    loop {
        if let Some(name) = world.get::<Name>(at).map(|name| name.0.clone())
            && is_room(&name)
        {
            return (!world.resource::<Opened>().is_open(&name)).then_some(name);
        }
        at = world.get::<ChildOf>(at).map(ChildOf::parent)?;
    }
}

/// What the player may make: the secrets they have found and the products
/// they have earned, read together.
///
/// **The one question, asked of both holders.** `Recipes::matching` and its
/// siblings used to take `&Learned`; a second parameter for `Opened` at every
/// site would be two expressions of one rule, which is the defect §19 records
/// most often.
#[derive(Debug, Clone, Copy)]
pub struct Known<'a> {
    learned: &'a Learned,
    opened: &'a Opened,
}

impl<'a> Known<'a> {
    /// Read both holders.
    #[must_use]
    pub const fn new(learned: &'a Learned, opened: &'a Opened) -> Self {
        Self { learned, opened }
    }

    /// Whether the player can make `name`.
    ///
    /// A name no recipe hides is always known; a secret is known once found;
    /// a gated product is known once its station is reached.
    #[must_use]
    pub fn knows(&self, recipes: &Recipes, name: &str) -> bool {
        if recipes.is_gated(name) {
            return self.opened.has(&recipe_key(name));
        }
        self.learned.knows(recipes, name)
    }
}

/// What the player may make, read off the world.
#[must_use]
pub fn known(world: &World) -> Known<'_> {
    Known::new(world.resource::<Learned>(), world.resource::<Opened>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_reads_back_as_itself() {
        for text in ["domain:archive", "recipe:warding", "charm:whetted", "siege"] {
            let key = Key::parse(text).unwrap_or_else(|| panic!("{text} did not parse"));
            assert_eq!(key.to_string(), text);
        }
    }

    #[test]
    fn a_key_of_no_kind_does_not_parse() {
        for text in ["wall", "domain:", "room:archive", ":archive", ""] {
            assert!(Key::parse(text).is_none(), "{text:?} parsed");
        }
    }

    #[test]
    fn an_open_tower_holds_every_room_every_gated_product_and_every_charm() {
        let recipes = Recipes::builtin();
        let charms = Charms::builtin();
        let opened = Opened::all(&recipes, &charms);
        for domain in super::super::DOMAINS {
            assert!(opened.is_open(domain), "{domain} is shut");
        }
        assert!(opened.is_open(super::super::siege::BAILEY));
        for product in recipes.gated() {
            assert!(opened.has(&recipe_key(product)), "{product} is shut");
        }
        for charm in charms.names() {
            assert!(opened.has(&charm_key(charm)), "{charm} is shut");
        }
        assert!(opened.has(SIEGE));
    }

    #[test]
    fn a_fresh_game_is_a_laboratory_and_what_no_station_opens() {
        let recipes = Recipes::builtin();
        let charms = Charms::builtin();
        let curve = Progression::builtin();
        let start = Opened::start(&recipes, &charms, &curve);
        assert!(start.is_open("laboratory"), "the laboratory is shut");
        for domain in [
            "archive",
            "lens",
            "grimoire",
            "sanctum",
            "menagerie",
            "forge",
        ] {
            assert!(!start.is_open(domain), "{domain} is open at the start");
        }
        assert!(
            !start.is_open(super::super::siege::BAILEY),
            "the bailey is open"
        );
        assert!(!start.has(SIEGE), "the wall is armed at the start");
        assert!(
            start.has(&charm_key("hurried")),
            "the flagship charm is shut"
        );
        assert!(
            !start.has(&charm_key("whetted")),
            "a station's charm is open"
        );
        assert!(
            !start.has(&recipe_key("warding")),
            "a station's recipe is open"
        );
    }

    #[test]
    fn opening_says_whether_it_changed_anything() {
        let mut opened = Opened::default();
        assert!(opened.insert("domain:archive"), "a shut room did not open");
        assert!(
            !opened.insert("domain:archive"),
            "an open room opened again"
        );
        assert!(opened.is_open("archive"));
    }
}
