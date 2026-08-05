//! Conspiracy (CNS) — the draft-matters cards (CR 905.2). Their draft-time
//! text lives in `crabomination::draft::DraftPod`; what's here is the half
//! that functions in the game the draft produced. Tests in
//! `classic_sets/cns`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::{DraftNoteAgg, Effect, ManaPayload, PlayerRef, Value};
use crate::mana::{ManaCost, cost, generic};

fn construct(name: &'static str, c: ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Aether Searcher — free-casts a card whose name it noted during the draft.
pub fn aether_searcher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            // The printed "you may" rides the free cast's own yes/no prompt.
            effect: Effect::SearchAndCastFree {
                filter: R::NameNotedForSource,
                include_hand: true,
            },
        }],
        ..construct("Aether Searcher", cost(&[generic(7)]), 6, 4)
    }
}

/// Agent of Acquisitions — drafts a whole pack; a plain 2/1 in the game.
pub fn agent_of_acquisitions() -> CardDefinition {
    construct("Agent of Acquisitions", cost(&[generic(2)]), 2, 1)
}

/// Cogwork Grinder — one +1/+1 counter per card it removed from the draft.
pub fn cogwork_grinder() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::DraftNoteNumber { agg: DraftNoteAgg::Sum },
        )),
        ..construct("Cogwork Grinder", cost(&[generic(6)]), 0, 0)
    }
}

/// Cogwork Librarian — trades itself for an extra draft pick; a 3/3 in the game.
pub fn cogwork_librarian() -> CardDefinition {
    construct("Cogwork Librarian", cost(&[generic(4)]), 3, 3)
}

/// Lore Seeker — adds a booster to the draft; a 2/2 in the game.
pub fn lore_seeker() -> CardDefinition {
    construct("Lore Seeker", cost(&[generic(2)]), 2, 2)
}

/// Lurking Automaton — sized by the highest pick number it noted.
pub fn lurking_automaton() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::DraftNoteNumber { agg: DraftNoteAgg::Max },
        )),
        ..construct("Lurking Automaton", cost(&[generic(5)]), 0, 0)
    }
}

/// Paliano, the High City — taps for one of the three colors noted at draft.
pub fn paliano_the_high_city() -> CardDefinition {
    CardDefinition {
        name: "Paliano, the High City",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::DraftNotedColorOfSource,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Whispergear Sneak — peeks at a pack mid-draft; a 1/1 in the game.
pub fn whispergear_sneak() -> CardDefinition {
    construct("Whispergear Sneak", cost(&[generic(1)]), 1, 1)
}
