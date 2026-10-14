//! Commander: one-card gaps of EDHREC's commanders ranked 5501-6500 (the
//! 2026-10-14 scan), ninth file. Tests in `tests/recent_b/cmdr_rank6k_i.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, SelectionRequirement as R,
    Subtypes, Supertype, Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, PlayerRef, Selector};
use crate::mana::{cost, g, generic};

/// Mikey & Mona, Mutant Sitters — {2}{G} 2/2 legendary Mutant Ninja Turtle
/// Lizard. ETB, one or both, each mode targeting a different player: target
/// player chooses a creature they control and puts two +1/+1 counters on it;
/// target player returns a creature or land card from their graveyard to
/// their hand. (Residual: "a different player" per mode is not enforced.)
pub fn mikey_mona_mutant_sitters() -> CardDefinition {
    CardDefinition {
        name: "Mikey & Mona, Mutant Sitters",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle, CreatureType::Lizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::ChooseN {
            picks: vec![0, 1],
            modes: vec![
                Effect::TargetPlayerThen {
                    filter: R::Player,
                    then: Box::new(Effect::AsPlayer {
                        who: PlayerRef::Target(0),
                        body: Box::new(Effect::ChooseOneAmong {
                            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                            chooser: PlayerRef::You,
                            chosen: Box::new(Effect::AddCounter {
                                what: Selector::SeparatedPile { chosen: true },
                                kind: CounterType::PlusOnePlusOne,
                                amount: Value::Const(2),
                            }),
                            other: Box::new(Effect::Noop),
                        }),
                    }),
                },
                Effect::TargetPlayerThen {
                    filter: R::Player,
                    then: Box::new(Effect::AsPlayer {
                        who: PlayerRef::Target(0),
                        body: Box::new(Effect::ReturnGraveyardCardsToHand {
                            filter: R::Creature.or(R::Land),
                            max: Value::ONE,
                        }),
                    }),
                },
            ],
        })],
        ..Default::default()
    }
}
