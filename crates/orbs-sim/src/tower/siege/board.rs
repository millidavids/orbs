//! What the rampart's board draws of a [`Siege`] (§5.1).

use super::{Area, ENEMY, GARRISON, Siege, VIGOUR};

impl Siege {
    /// What the board draws.
    ///
    /// The names and the intent travel with it, so the painter need not know
    /// what a band is called — `orbs-render` may never depend on `orbs-sim`
    /// (§19). The odds go too, because §5.1 is *show the odds before the
    /// commitment*. `pool` and `flowing` are handed in, for the reason `pledge`
    /// takes the pool: both are the tower's, not the siege's.
    #[must_use]
    pub fn view(
        &self,
        tally: String,
        pool: u32,
        flowing: Vec<(String, u32)>,
    ) -> orbs_render::Rampart {
        orbs_render::Rampart {
            garrison: orbs_render::SiegeSide {
                name: GARRISON,
                troops: self.garrison.count,
                vigour: self.garrison.vigour,
                // `mustered`, not the current count: against `count` the bar
                // never fell below ~89%, so a line cut from six to two drew
                // full one round before it routed.
                full: self.full(),
                chance: self.garrison_roll().chance(),
            },
            enemy: orbs_render::SiegeSide {
                name: ENEMY,
                troops: self.enemy.count,
                vigour: self.enemy.vigour,
                full: self.arrived * VIGOUR,
                chance: self.enemy_roll().chance(),
            },
            turns: self.turns,
            // The whole allocation. Three dice against four rows, so one is
            // always empty — which one is the decision the board makes visible.
            areas: Area::ALL
                .into_iter()
                .map(|area| orbs_render::Allocation {
                    name: area.word(),
                    dice: self
                        .pledged(area)
                        .into_iter()
                        .map(|die| die.word().to_owned())
                        .collect(),
                    range: self.range(area),
                    moot: !area.answers(self.intent),
                })
                .collect(),
            // Everything held, with its price: a die that vanished when you
            // could not pay for it would hide the round where the decision is
            // hardest. `affordable()` is the spell's narrower view.
            coffer: self
                .coffer()
                .into_iter()
                .map(|die| (die.word().to_owned(), super::cost_of(die)))
                .collect(),
            quintessence: pool,
            flowing,
            intent: self.intent.word().to_owned(),
            tally,
        }
    }
}
