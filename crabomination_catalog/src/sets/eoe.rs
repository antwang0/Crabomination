//! Edge of Eternities — Exhaust (CR 702.177). "Exhaust — [Cost]: [Effect]"
//! means "[Cost]: [Effect]. Activate only once" (per game). Modeled via the
//! `ActivatedAbility.exhaust` flag + `CardInstance.exhausted_abilities`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword, Subtypes,
    TokenDefinition,
};
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::mana::{cost, generic};

/// Camera Launcher — {3} Artifact Creature — Construct 2/2. "Exhaust — {3}:
/// Put a +1/+1 counter on this creature. Create a 1/1 colorless Thopter
/// artifact creature token with flying."
pub fn camera_launcher() -> CardDefinition {
    let thopter = TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thopter],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Camera Launcher",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: thopter },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
