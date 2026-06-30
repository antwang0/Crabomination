//! Bloomburrow **Gift** cards (CR 702.165). Each spell may promise its gift to
//! an opponent as it's cast (`GameAction::CastGift`); if promised, the opponent
//! receives the gift and the spell resolves its enhanced `gifted_effect`
//! instead of the printed base `effect`. Tracked in `DECK_FEATURES.md`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Gift, Keyword, SelectionRequirement, Selector,
    Subtypes, TokenDefinition, Value,
};
use crate::effect::shortcut::{each_your_creature, target_filtered};
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{cost, generic, g, r, u, w, Color};

/// Opponent draws one card — the "Gift a card" payload.
fn opponent_draws_one() -> Effect {
    Effect::Draw { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(1) }
}

/// Opponent creates a Food token — the "Gift a Food" payload.
fn opponent_food() -> Effect {
    Effect::CreateToken {
        who: PlayerRef::EachOpponent,
        count: Value::Const(1),
        definition: crabomination_base::tokens::food_token(),
    }
}

/// A tapped 1/1 blue Fish — the gift on the Bloomburrow blue/green cards.
fn tapped_fish_token() -> TokenDefinition {
    TokenDefinition {
        name: "Fish".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fish],
            ..Default::default()
        },
        tapped: true,
        ..Default::default()
    }
}

/// Crumb and Get It — {W} Instant. Gift a Food. Target creature you control
/// gets +2/+2; if the gift was promised, it also gains indestructible.
pub fn crumb_and_get_it() -> CardDefinition {
    let pump = Effect::PumpPT {
        what: target_filtered(SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou)),
        power: Value::Const(2),
        toughness: Value::Const(2),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Crumb and Get It",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: pump.clone(),
        gift: Some(Box::new(Gift {
            label: "a Food",
            gifted_effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::EachOpponent,
                    count: Value::Const(1),
                    definition: crabomination_base::tokens::food_token(),
                },
                pump,
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ]),
        })),
        ..Default::default()
    }
}

/// Blooming Blast — {1}{R} Instant. Gift a Treasure. Deal 2 damage to target
/// creature; if the gift was promised, also deal 3 damage to that creature's
/// controller.
pub fn blooming_blast() -> CardDefinition {
    let bolt = Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(2) };
    CardDefinition {
        name: "Blooming Blast",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: bolt.clone(),
        gift: Some(Box::new(Gift {
            label: "a Treasure",
            gifted_effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::EachOpponent,
                    count: Value::Const(1),
                    definition: crabomination_base::tokens::treasure_token(),
                },
                bolt,
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(3),
                },
            ]),
        })),
        ..Default::default()
    }
}

/// Longstalk Brawl — {G} Sorcery. Gift a tapped Fish. Choose target creature
/// you control and target creature you don't control. If the gift was promised,
/// put a +1/+1 counter on the creature you control. Then they fight.
pub fn longstalk_brawl() -> CardDefinition {
    let fight = Effect::Fight {
        attacker: Selector::TargetFiltered {
            slot: 0,
            filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        },
        defender: Selector::TargetFiltered {
            slot: 1,
            filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
        },
    };
    CardDefinition {
        name: "Longstalk Brawl",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: fight.clone(),
        gift: Some(Box::new(Gift {
            label: "a tapped Fish",
            gifted_effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::EachOpponent,
                    count: Value::Const(1),
                    definition: tapped_fish_token(),
                },
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                fight,
            ]),
        })),
        ..Default::default()
    }
}

/// Into the Flood Maw — {U} Instant. Gift a tapped Fish. Return target creature
/// an opponent controls to its owner's hand; if the gift was promised, instead
/// return target nonland permanent an opponent controls.
pub fn into_the_flood_maw() -> CardDefinition {
    CardDefinition {
        name: "Into the Flood Maw",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
        gift: Some(Box::new(Gift {
            label: "a tapped Fish",
            gifted_effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::EachOpponent,
                    count: Value::Const(1),
                    definition: tapped_fish_token(),
                },
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Permanent
                            .and(SelectionRequirement::Nonland)
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
            ]),
        })),
        ..Default::default()
    }
}

/// Long River's Pull — {U}{U} Instant. Gift a card. Counter target creature
/// spell; if the gift was promised, instead counter target spell.
pub fn long_rivers_pull() -> CardDefinition {
    CardDefinition {
        name: "Long River's Pull",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack
                    .and(SelectionRequirement::HasCardType(CardType::Creature)),
            ),
        },
        gift: Some(Box::new(Gift {
            label: "a card",
            gifted_effect: Effect::Seq(vec![
                opponent_draws_one(),
                Effect::CounterSpell {
                    what: target_filtered(SelectionRequirement::IsSpellOnStack),
                },
            ]),
        })),
        ..Default::default()
    }
}

/// Dawn's Truce — {1}{W} Instant. Gift a card. Permanents you control gain
/// hexproof; if the gift was promised, they also gain indestructible. (The
/// "you" also gaining hexproof is omitted.)
pub fn dawns_truce() -> CardDefinition {
    let your_perms = || Selector::ControlledBy {
        who: PlayerRef::You,
        filter: SelectionRequirement::Permanent,
    };
    let grant = |kw| Effect::GrantKeyword { what: your_perms(), keyword: kw, duration: Duration::EndOfTurn };
    CardDefinition {
        name: "Dawn's Truce",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: grant(Keyword::Hexproof),
        gift: Some(Box::new(Gift {
            label: "a card",
            gifted_effect: Effect::Seq(vec![
                opponent_draws_one(),
                grant(Keyword::Hexproof),
                grant(Keyword::Indestructible),
            ]),
        })),
        ..Default::default()
    }
}

/// Wildfire Howl — {1}{R}{R} Sorcery. Gift a card. Deal 2 damage to each
/// creature; if the gift was promised, also deal 1 damage to any target.
pub fn wildfire_howl() -> CardDefinition {
    let sweep = || Effect::ForEach {
        selector: Selector::EachPermanent(SelectionRequirement::Creature),
        body: Box::new(Effect::DealDamage { to: Selector::TriggerSource, amount: Value::Const(2) }),
    };
    CardDefinition {
        name: "Wildfire Howl",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: sweep(),
        gift: Some(Box::new(Gift {
            label: "a card",
            gifted_effect: Effect::Seq(vec![
                opponent_draws_one(),
                Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(1) },
                sweep(),
            ]),
        })),
        ..Default::default()
    }
}

/// Mind Spiral — {4}{U} Sorcery. Gift a tapped Fish. Target player draws three
/// cards; if the gift was promised, tap target creature an opponent controls
/// and put a stun counter on it.
pub fn mind_spiral() -> CardDefinition {
    let draw3 = || Effect::Draw {
        who: target_filtered(SelectionRequirement::Player),
        amount: Value::Const(3),
    };
    let opp_creature = || {
        Selector::TargetFiltered {
            slot: 1,
            filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
        }
    };
    CardDefinition {
        name: "Mind Spiral",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Sorcery],
        effect: draw3(),
        gift: Some(Box::new(Gift {
            label: "a tapped Fish",
            gifted_effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::EachOpponent,
                    count: Value::Const(1),
                    definition: tapped_fish_token(),
                },
                draw3(),
                Effect::Tap { what: opp_creature() },
                Effect::AddCounter {
                    what: opp_creature(),
                    kind: crate::card::CounterType::Stun,
                    amount: Value::Const(1),
                },
            ]),
        })),
        ..Default::default()
    }
}

/// Peerless Recycling — {1}{G} Instant. Gift a card. Return target permanent
/// card from your graveyard to your hand; if the gift was promised, return two.
pub fn peerless_recycling() -> CardDefinition {
    let ret = |slot| Effect::Move {
        what: Selector::TargetFiltered {
            slot,
            filter: SelectionRequirement::InYourGraveyard.and(SelectionRequirement::Permanent),
        },
        to: ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        name: "Peerless Recycling",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: ret(0),
        gift: Some(Box::new(Gift {
            label: "a card",
            gifted_effect: Effect::Seq(vec![opponent_draws_one(), ret(0), ret(1)]),
        })),
        ..Default::default()
    }
}

/// Valley Rally — {2}{R} Instant. Gift a Food. Creatures you control get +2/+0;
/// if the gift was promised, target creature you control also gains first strike.
pub fn valley_rally() -> CardDefinition {
    let pump = Effect::PumpPT {
        what: each_your_creature(),
        power: Value::Const(2),
        toughness: Value::Const(0),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Valley Rally",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: pump.clone(),
        gift: Some(Box::new(Gift {
            label: "a Food",
            gifted_effect: Effect::Seq(vec![
                opponent_food(),
                pump,
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
        })),
        ..Default::default()
    }
}
