use crate::card::{CardDefinition, CardType, Keyword};
use crate::effect::shortcut::{deal, target};
use crate::effect::{Effect, PlayerRef, Value};
use crate::mana::{cost, g, generic, r};
use crabomination_base::tokens::eldrazi_scion_token;

/// Call the Scions — {2}{G} Devoid Sorcery. Create two 1/1 Eldrazi Scions.
pub fn call_the_scions() -> CardDefinition {
    CardDefinition {
        name: "Call the Scions",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Devoid],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: eldrazi_scion_token(),
        },
        ..Default::default()
    }
}

/// Reality Hemorrhage — {1}{R} Devoid Instant. Deals 2 damage to any target.
pub fn reality_hemorrhage() -> CardDefinition {
    CardDefinition {
        name: "Reality Hemorrhage",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: deal(2, target()),
        ..Default::default()
    }
}

/// Touch of the Void — {2}{R} Devoid Sorcery. Deals 3 damage to any target.
/// (The "if a creature dies this turn, exile it" rider is dropped.)
pub fn touch_of_the_void() -> CardDefinition {
    CardDefinition {
        name: "Touch of the Void",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Devoid],
        effect: deal(3, target()),
        ..Default::default()
    }
}
