//! Return to Ravnica (RTR) gap wave 13: Slaughter Games (name-exile across all
//! zones) and Guild Feud (dueling top-three reveal). Tests in `classic_sets/rtr`.

use crate::card::{
    CardDefinition, CardType, EventKind, EventScope, EventSpec, Keyword, TriggeredAbility,
};
use crate::effect::Effect;
use crate::game::TurnStep;
use crate::mana::{b, cost, generic, r};

/// Slaughter Games — {2}{B}{R} Sorcery that can't be countered. Choose a
/// nonland card name; exile every card with that name from target opponent's
/// graveyard, hand, and library, then they shuffle.
pub fn slaughter_games() -> CardDefinition {
    CardDefinition {
        name: "Slaughter Games",
        cost: cost(&[generic(2), b(), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::NameCardExileMatchingAllZones,
        ..Default::default()
    }
}

/// Guild Feud — {5}{R} Enchantment. At your upkeep, target opponent reveals
/// three, may deploy a creature (rest to their graveyard); you do the same; if
/// two creatures enter, they fight.
pub fn guild_feud() -> CardDefinition {
    CardDefinition {
        name: "Guild Feud",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::GuildFeud,
        }],
        ..Default::default()
    }
}
