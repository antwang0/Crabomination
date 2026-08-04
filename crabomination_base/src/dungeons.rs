//! CR 309 — Dungeon cards. A dungeon lives outside the game until a player
//! ventures (CR 701.49); its rooms resolve as they're entered and the dungeon
//! completes when the final room's ability has resolved.
//!
//! Room abilities resolve inline (the printed rooms are triggered abilities —
//! the stack round-trip is elided) with auto-pick selectors standing in for
//! their targets. Tomb of Annihilation's two "lose 2 life unless you
//! discard/sacrifice" rooms are modeled as the flat life loss.

use crate::card::{CardType, CounterType, CreatureType, MayPlayDuration, Subtypes, Supertype, TokenDefinition};
use crate::card::SelectionRequirement as R;
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::mana::Color;

pub struct DungeonRoom {
    pub name: &'static str,
    pub effect: Effect,
    /// Indices of the rooms this one leads to; empty = final room.
    pub next: &'static [u8],
}

pub struct DungeonDefinition {
    pub name: &'static str,
    pub rooms: Vec<DungeonRoom>,
}

/// The three AFR dungeons, in venture-choice order. Undercity is excluded —
/// CR 309.7 / its own text: you can only enter it by "venturing into Undercity"
/// (the initiative, CR 726).
pub fn dungeon_names() -> [&'static str; 3] {
    ["Lost Mine of Phandelver", "Dungeon of the Mad Mage", "Tomb of Annihilation"]
}

/// The name of the initiative's dungeon (CR 726.2).
pub const UNDERCITY: &str = "Undercity";

pub fn dungeon_by_name(name: &str) -> Option<DungeonDefinition> {
    match name {
        "Lost Mine of Phandelver" => Some(lost_mine_of_phandelver()),
        "Dungeon of the Mad Mage" => Some(dungeon_of_the_mad_mage()),
        "Tomb of Annihilation" => Some(tomb_of_annihilation()),
        UNDERCITY => Some(undercity()),
        _ => None,
    }
}

/// CR 726 — the initiative's dungeon (Baldur's Gate).
pub fn undercity() -> DungeonDefinition {
    DungeonDefinition {
        name: UNDERCITY,
        rooms: vec![
            DungeonRoom {
                name: "Secret Entrance",
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand,
                    to: crate::effect::ZoneDest::Hand(PlayerRef::You),
                },
                next: &[1, 2],
            },
            DungeonRoom {
                name: "Forge",
                effect: Effect::AddCounter {
                    what: your_creature(),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                next: &[3, 4],
            },
            DungeonRoom {
                name: "Lost Well",
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
                next: &[4, 5],
            },
            DungeonRoom {
                name: "Trap!",
                effect: Effect::LoseLife {
                    who: Selector::one_of(Selector::Player(PlayerRef::EachOpponent)),
                    amount: Value::Const(5),
                },
                next: &[6],
            },
            DungeonRoom {
                name: "Arena",
                effect: Effect::Goad { what: opp_creature() },
                next: &[6, 7],
            },
            DungeonRoom {
                name: "Stash",
                effect: crate::effect::shortcut::mint_treasures(1),
                next: &[7],
            },
            DungeonRoom {
                name: "Archives",
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                next: &[8],
            },
            DungeonRoom {
                name: "Catacombs",
                effect: mint(undercity_skeleton_token(), 1),
                next: &[8],
            },
            DungeonRoom {
                name: "Throne of the Dead Three",
                effect: Effect::RevealTopNPutMatchingToBattlefield {
                    who: PlayerRef::You,
                    count: Value::Const(10),
                    filter: R::Creature,
                },
                next: &[],
            },
        ],
    }
}

/// Catacombs' 4/1 black Skeleton with menace.
fn undercity_skeleton_token() -> TokenDefinition {
    TokenDefinition {
        power: 4,
        toughness: 1,
        keywords: vec![crate::card::Keyword::Menace],
        ..skeleton_1_1_token()
    }
}

fn your_creature() -> Selector {
    Selector::one_of(Selector::EachPermanent(R::Creature.and(R::ControlledByYou)))
}

fn opp_creature() -> Selector {
    Selector::one_of(Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)))
}

fn goblin_1_1_token() -> TokenDefinition {
    TokenDefinition {
        name: "Goblin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        ..Default::default()
    }
}

fn skeleton_1_1_token() -> TokenDefinition {
    TokenDefinition {
        name: "Skeleton".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Skeleton], ..Default::default() },
        ..Default::default()
    }
}

/// The Atropal — legendary 4/4 black God Horror with deathtouch.
fn atropal_token() -> TokenDefinition {
    TokenDefinition {
        name: "The Atropal".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        supertypes: vec![Supertype::Legendary],
        keywords: vec![crate::card::Keyword::Deathtouch],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::God, CreatureType::Horror],
            ..Default::default()
        },
        ..Default::default()
    }
}

fn mint(token: TokenDefinition, n: i32) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: token }
}

pub fn lost_mine_of_phandelver() -> DungeonDefinition {
    DungeonDefinition {
        name: "Lost Mine of Phandelver",
        rooms: vec![
            DungeonRoom {
                name: "Cave Entrance",
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
                next: &[1, 2],
            },
            DungeonRoom { name: "Goblin Lair", effect: mint(goblin_1_1_token(), 1), next: &[3, 4] },
            DungeonRoom {
                name: "Mine Tunnels",
                effect: mint(crate::tokens::treasure_token(), 1),
                next: &[4, 5],
            },
            DungeonRoom {
                name: "Storeroom",
                effect: Effect::AddCounter {
                    what: your_creature(),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                next: &[6],
            },
            DungeonRoom {
                name: "Dark Pool",
                effect: Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(1) },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
                ]),
                next: &[6],
            },
            DungeonRoom {
                name: "Fungi Cavern",
                effect: Effect::PumpPT {
                    what: opp_creature(),
                    power: Value::Const(-4),
                    toughness: Value::Const(0),
                    duration: Duration::UntilYourNextUntap,
                },
                next: &[6],
            },
            DungeonRoom {
                name: "Temple of Dumathoin",
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                next: &[],
            },
        ],
    }
}

pub fn dungeon_of_the_mad_mage() -> DungeonDefinition {
    DungeonDefinition {
        name: "Dungeon of the Mad Mage",
        rooms: vec![
            DungeonRoom {
                name: "Yawning Portal",
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
                next: &[1],
            },
            DungeonRoom {
                name: "Dungeon Level",
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
                next: &[2, 3],
            },
            DungeonRoom {
                name: "Goblin Bazaar",
                effect: mint(crate::tokens::treasure_token(), 1),
                next: &[4],
            },
            DungeonRoom { name: "Twisted Caverns", effect: Effect::Detain { what: opp_creature() }, next: &[4] },
            DungeonRoom {
                name: "Lost Level",
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
                next: &[5, 6],
            },
            DungeonRoom {
                name: "Runestone Caverns",
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    duration: MayPlayDuration::WhileExiled,
                    pay_any_color: false,
                    pay_own_cost: false,
                    uncast_penalty: None,
                },
                next: &[7],
            },
            DungeonRoom { name: "Muiral's Graveyard", effect: mint(skeleton_1_1_token(), 2), next: &[7] },
            DungeonRoom {
                name: "Deep Mines",
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(3) },
                next: &[8],
            },
            // "Draw three cards and reveal them. You may cast one without
            // paying its cost" — the free cast is approximated to the draws.
            DungeonRoom {
                name: "Mad Wizard's Lair",
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                next: &[],
            },
        ],
    }
}

pub fn tomb_of_annihilation() -> DungeonDefinition {
    let each = || Selector::Player(PlayerRef::EachPlayer);
    DungeonDefinition {
        name: "Tomb of Annihilation",
        rooms: vec![
            DungeonRoom {
                name: "Trapped Entry",
                effect: Effect::LoseLife { who: each(), amount: Value::Const(1) },
                next: &[1, 3],
            },
            DungeonRoom {
                name: "Veils of Fear",
                effect: Effect::LoseLife { who: each(), amount: Value::Const(2) },
                next: &[2],
            },
            DungeonRoom {
                name: "Sandfall Cell",
                effect: Effect::LoseLife { who: each(), amount: Value::Const(2) },
                next: &[4],
            },
            DungeonRoom {
                name: "Oubliette",
                effect: Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                    Effect::Sacrifice { who: Selector::You, count: Value::Const(1), filter: R::Creature },
                    Effect::Sacrifice { who: Selector::You, count: Value::Const(1), filter: R::Artifact },
                    Effect::Sacrifice { who: Selector::You, count: Value::Const(1), filter: R::Land },
                ]),
                next: &[4],
            },
            DungeonRoom { name: "Cradle of the Death God", effect: mint(atropal_token(), 1), next: &[] },
        ],
    }
}
