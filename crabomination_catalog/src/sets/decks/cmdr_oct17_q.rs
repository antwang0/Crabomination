//! Commander: 2026-10-17 (`3chree`) one-card-gap closers — each the last
//! missing card of one or more unseated decks. Tests in
//! `tests/recent_b/cmdr_oct17_q.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, ExileReturnZone, MayPlayDuration, Predicate, SelectionRequirement as R, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, cost, g, generic, r, w, x};

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}




/// Dust of Moments — {2}{W} instant. Choose one: remove two time counters
/// from each permanent and each suspended card; or put two time counters on
/// each permanent with a time counter on it and each suspended card.
pub fn dust_of_moments() -> CardDefinition {
    let mode = |add| Effect::TimeCountersOnEachPermanentAndSuspended { add, amount: 2 };
    spell("Dust of Moments", cost(&[generic(2), w()]), CardType::Instant, Effect::ChooseMode(vec![mode(false), mode(true)]))
}

/// On Thin Ice — {W} snow Aura, enchant snow land you control. When it
/// enters, exile target creature an opponent controls until it leaves.
pub fn on_thin_ice() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..spell(
            "On Thin Ice",
            cost(&[w()]),
            CardType::Enchantment,
            Effect::Attach {
                what: Selector::This,
                to: target_filtered(R::Land.and(R::IsSnow).and(R::ControlledByYou)),
            },
        )
    }
}


/// Dawn-Blessed Pennant — {1} artifact. As it enters, choose Elemental, Elf,
/// Faerie, Giant, Goblin, Kithkin, Merfolk, or Treefolk. Whenever a permanent
/// you control of that type enters, gain 1 life. {2}, {T}, sacrifice: return
/// target card of the chosen type from your graveyard to your hand.
pub fn dawn_blessed_pennant() -> CardDefinition {
    use CreatureType::*;
    CardDefinition {
        as_enters_effect: Some(Effect::NameCreatureTypeAmong {
            what: Selector::This,
            options: vec![Elemental, Elf, Faerie, Giant, Goblin, Kithkin, Merfolk, Treefolk],
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsSourceChosenCreatureType },
            ),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::IsSourceChosenCreatureType.from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..spell("Dawn-Blessed Pennant", cost(&[generic(1)]), CardType::Artifact, Effect::Noop)
    }
}

/// The Flame of Keld — {1}{R} Saga. I: discard your hand. II: draw two. III:
/// this turn, a red source you control deals +2 damage to any permanent or
/// player (CR 614.1a).
pub fn the_flame_of_keld() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, Effect::Discard { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You), random: false }),
            (2, Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
            (3, Effect::YourRedSourcesDealExtraDamageThisTurn { amount: 2 }),
        ],
        ..spell("The Flame of Keld", cost(&[generic(1), r()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Celestial Reunion — {X}{G} sorcery. You may choose a creature type and
/// behold two creatures of that type as an additional cost. Search for a
/// creature card with mana value X or less; it goes onto the battlefield if
/// the cost was paid and it has the chosen type, else into your hand.
pub fn celestial_reunion() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::BeholdOfChosenType { count: 2 }),
        ..spell(
            "Celestial Reunion",
            cost(&[x(), g()]),
            CardType::Sorcery,
            Effect::Search {
                who: PlayerRef::You,
                filter: R::Creature.and(R::ManaValueAtMostXFromCost),
                to: ZoneDest::IfCard {
                    filter: Box::new(R::IsSourceChosenCreatureType),
                    then: Box::new(ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false }),
                    else_: Box::new(ZoneDest::Hand(PlayerRef::You)),
                },
            },
        )
    }
}


/// March of Reckless Joy — {X}{R} instant. You may exile any number of red
/// cards from your hand as you cast it; {2} less for each. Exile the top X
/// cards of your library; you may play up to two of them until the end of
/// your next turn.
pub fn march_of_reckless_joy() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::card::StaticAbility {
            description: "As an additional cost to cast this spell, you may exile any number of red cards from your hand. This spell costs {2} less to cast for each card exiled this way.",
            effect: crate::effect::StaticEffect::ExileFromHandCostReduction { per: 2, filter: R::HasColor(Color::Red) },
        }],
        ..spell(
            "March of Reckless Joy",
            cost(&[x(), r()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::XFromCost,
                    duration: MayPlayDuration::EndOfControllersNextTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
                Effect::PlaysAmongGranted { what: Selector::ExiledThisResolution { filter: R::Any }, plays: 2 },
            ]),
        )
    }
}
