//! Bloomburrow (BLB) gap batch — an Offspring attack-payoff Knight and an
//! X-cost token-doubler. Tests in `tests/recent223.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, Subtypes,
    TokenDefinition,
};
use crate::effect::shortcut::on_you_attack;
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value, ZoneRef};
use crate::mana::{Color, cost, g, generic, w, x};

fn rabbit_1_1() -> TokenDefinition {
    TokenDefinition {
        name: "Rabbit".to_string(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Warren Warleader — {2}{W}{W} 4/4 Rabbit Knight. Offspring {2}. Whenever you
/// attack, choose one — make a 1/1 Rabbit tapped and attacking, or attacking
/// creatures you control get +1/+1 until end of turn.
pub fn warren_warleader() -> CardDefinition {
    CardDefinition {
        name: "Warren Warleader",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Offspring(cost(&[generic(2)]))],
        triggered_abilities: vec![on_you_attack(Effect::ChooseMode(vec![
            Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(rabbit_1_1()),
                cleanup: Default::default(),
            },
            Effect::PumpPT {
                what: Selector::EachMatching {
                    zone: ZoneRef::Battlefield,
                    filter: R::Creature.and(R::ControlledByYou).and(R::IsAttacking),
                },
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// For the Common Good — {X}{X}{G} Sorcery. Create X token copies of target
/// token you control; tokens you control gain indestructible until your next
/// turn; gain 1 life for each token you control.
pub fn for_the_common_good() -> CardDefinition {
    let your_tokens = || Selector::EachMatching {
        zone: ZoneRef::Battlefield,
        filter: R::IsToken.and(R::ControlledByYou),
    };
    CardDefinition {
        name: "For the Common Good",
        cost: cost(&[x(), x(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::XFromCost,
                source: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::IsToken.and(R::ControlledByYou),
                },
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            },
            Effect::GrantKeyword {
                what: your_tokens(),
                keyword: Keyword::Indestructible,
                duration: Duration::UntilNextTurn,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::count(your_tokens()),
            },
        ]),
        ..Default::default()
    }
}
