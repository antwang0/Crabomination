//! Counters on battlefield permanents: a pile of 3-D coins per counter kind
//! down the card's left edge, the count printed on the pile's top coin and
//! the kind named beside it.
//!
//! The column sits over the left of the art box, clear of the name bar, the
//! P/T box and every corner badge. A pile grows a coin per counter up to
//! [`MAX_PILE`], so "a few" and "a lot" read apart at a glance; the number
//! says exactly how many. The coins used to stack *across* the card face, a
//! coin-diameter per counter, so a Walking Ballista with eleven drew a
//! tower the height of the card over its art and name.
//!
//! A kind another overlay already reads gets no coin ([`board_counters`]):
//! a planeswalker's loyalty is the ◆ badge, a battle's defense the ◇ badge
//! (`pt_label`), a stun counter on a creature the "Stun N" status chip
//! (`keyword_label`).
//!
//! Coins are children of the card entity, rebuilt only when a card's piles
//! change, and a coin a pile gains drops onto it ([`CoinDrop`]); the labels
//! are screen-space nodes reprojected every frame, like `pt_label`'s
//! badges. The coin itself, a poker chip, is `coin_mesh`.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use crabomination::card::{CardId, CardType, CounterType};
use crabomination::net::PermanentView;

use crate::card::{BattlefieldCard, CARD_HEIGHT, CARD_THICKNESS, CARD_WIDTH, GameCardId};
use crate::net_plugin::CurrentView;

/// The material every chip shares, and each kind's chip
/// (`coin_mesh::chip_mesh`), made the first time that kind is shown.
#[derive(Resource)]
pub struct CounterCoinAssets {
    material: Handle<StandardMaterial>,
    /// The same, darkened, for the chips on a card focus dims
    /// (`systems::focus`).
    dimmed: Handle<StandardMaterial>,
    chips: HashMap<CounterType, Handle<Mesh>>,
}

impl CounterCoinAssets {
    /// The material a chip takes: the shared one, or its darkened twin on a
    /// dimmed card.
    pub fn material(&self, dimmed: bool) -> &Handle<StandardMaterial> {
        if dimmed { &self.dimmed } else { &self.material }
    }
}

/// Marker on each spawned coin-mesh entity, with its kind so a card's
/// current piles can be read back off its children.
#[derive(Component)]
pub struct CounterCoin {
    pub card_id: CardId,
    pub kind: CounterType,
}

/// Coin geometry, card-local. The card's face lies in XY with +Z out of the
/// front, which is world +Y once the card lies flat on the table, so a pile
/// grows along +Z — toward the camera, like chips on a card.
const COIN_RADIUS: f32 = 0.34;
const COIN_HEIGHT: f32 = 0.06;
const COIN_BASE_Z: f32 = CARD_THICKNESS / 2.0 + 0.01;
/// Coins in the tallest pile; the number on top carries the rest.
const MAX_PILE: u32 = 5;
/// The column: inset from the left edge, the first pile just under the name
/// bar, one pile per kind going down.
const COLUMN_X: f32 = -CARD_WIDTH / 2.0 + COIN_RADIUS + 0.14;
const COLUMN_TOP_Y: f32 = CARD_HEIGHT / 2.0 - 0.85;
const COLUMN_STEP: f32 = COIN_RADIUS * 2.0 + 0.12;

/// Coins in the pile for `count` counters.
fn pile_height(count: u32) -> u32 {
    count.clamp(1, MAX_PILE)
}

/// Card-local point on the card face under the pile in column slot `slot`.
fn pile_base(slot: usize) -> Vec3 {
    Vec3::new(COLUMN_X, COLUMN_TOP_Y - slot as f32 * COLUMN_STEP, 0.0)
}

/// Card-local centre of the top face of the pile in column slot `slot`.
fn pile_top(slot: usize, count: u32) -> Vec3 {
    Vec3::new(
        COLUMN_X,
        COLUMN_TOP_Y - slot as f32 * COLUMN_STEP,
        COIN_BASE_Z + pile_height(count) as f32 * COIN_HEIGHT,
    )
}

pub fn init_counter_coin_assets(commands: &mut Commands, materials: &mut ResMut<Assets<StandardMaterial>>) {
    let dimmed = StandardMaterial {
        base_color: crate::systems::focus::dimmed_tint(),
        ..super::coin_mesh::chip_material()
    };
    commands.insert_resource(CounterCoinAssets {
        material: materials.add(super::coin_mesh::chip_material()),
        dimmed: materials.add(dimmed),
        chips: HashMap::new(),
    });
}

/// A kind's chip colour: the deep shade of its tag's colour, so the chip
/// matches its tag. (The chips glowed their colour once, and washed out to
/// pastel under the number.)
fn coin_body(kind: CounterType) -> Color {
    shade(counter_label_color(kind), 0.5)
}

/// The count's ink on the chip's cream face: the kind's colour, darker still.
fn count_ink(kind: CounterType) -> Color {
    shade(counter_label_color(kind), 0.2)
}

fn shade(colour: Color, by: f32) -> Color {
    let c = colour.to_srgba();
    Color::srgb(c.red * by, c.green * by, c.blue * by)
}

/// A coin just added to a pile, falling onto it: card-local `rest_z` is
/// where it lands, `elapsed` starts below zero to stagger a pile's coins.
#[derive(Component)]
pub struct CoinDrop {
    rest_z: f32,
    elapsed: f32,
}

/// How far above its resting place a new coin starts, and how long it falls.
const DROP_HEIGHT: f32 = 0.9;
const DROP_SECS: f32 = 0.22;
/// Between one new coin of a pile and the next.
const DROP_STAGGER: f32 = 0.06;

/// Height above its resting place `elapsed` seconds into a fall: slow off
/// the top and fastest as it lands, as under gravity.
fn drop_lift(elapsed: f32) -> f32 {
    let k = (elapsed / DROP_SECS).clamp(0.0, 1.0);
    DROP_HEIGHT * (1.0 - k * k)
}

/// Bevy system: run each [`CoinDrop`] and drop it once the coin has landed.
pub fn animate_coin_drops(
    mut commands: Commands,
    time: Res<Time>,
    mut coins: Query<(Entity, &mut CoinDrop, &mut Transform)>,
) {
    for (e, mut drop, mut transform) in &mut coins {
        drop.elapsed += time.delta_secs();
        transform.translation.z = drop.rest_z + drop_lift(drop.elapsed);
        if drop.elapsed >= DROP_SECS {
            // `try_`: the coin goes with its card, which can leave mid-fall.
            commands.entity(e).try_remove::<CoinDrop>();
        }
    }
}

/// Column order: +1/+1 and −1/−1 first (the most read), then loyalty and
/// the other common kinds, the rest by name so the order holds still.
fn sort_key(kind: CounterType) -> (u8, &'static str) {
    let rank = match kind {
        CounterType::PlusOnePlusOne => 0,
        CounterType::MinusOneMinusOne => 1,
        CounterType::Loyalty => 2,
        CounterType::Charge => 3,
        CounterType::Stun => 4,
        CounterType::Time => 5,
        CounterType::Poison => 6,
        CounterType::Energy => 7,
        _ => 8,
    };
    (rank, counter_token(kind))
}

/// The counters `p` shows as coins, in column order: every kind with a
/// count, less the ones another overlay reads (module note).
pub(crate) fn board_counters(p: &PermanentView) -> Vec<(CounterType, u32)> {
    let creature = p.is_creature();
    let planeswalker = p.card_types.contains(&CardType::Planeswalker);
    let read_elsewhere = |kind: CounterType| match kind {
        CounterType::Loyalty => !creature && planeswalker,
        CounterType::Defense => {
            !creature && !planeswalker && p.card_types.contains(&CardType::Battle)
        }
        CounterType::Stun => crate::systems::keyword_label::has_status_strip(p),
        _ => false,
    };
    let mut kinds: Vec<(CounterType, u32)> = p
        .counters
        .iter()
        .filter(|&&(k, n)| n > 0 && !read_elsewhere(k))
        .copied()
        .collect();
    kinds.sort_by_key(|(k, _)| sort_key(*k));
    kinds
}

/// Reconcile the coin piles with the engine's battlefield view. A card
/// whose piles changed (a kind added or gone, or a pile's height moved) is
/// rebuilt; every other card's coins are left alone, so they stay parented
/// through tap and play animations.
pub fn sync_counter_coins(
    mut commands: Commands,
    view: Res<CurrentView>,
    assets: Option<ResMut<CounterCoinAssets>>,
    mut meshes: ResMut<Assets<Mesh>>,
    bf_cards: Query<(Entity, &GameCardId), With<BattlefieldCard>>,
    existing: Query<(Entity, &ChildOf, &CounterCoin)>,
    // Battlefield entities spawn via deferred commands, a frame after the
    // view change that caused them — their coins must still appear.
    added_bf: Query<(), Added<BattlefieldCard>>,
) {
    let Some(cv) = &view.0 else { return };
    let Some(mut assets) = assets else { return };
    // Coins ride their card, so this only needs to run when the counters
    // or the set of battlefield entities can have changed.
    if !view.is_changed() && added_bf.is_empty() {
        return;
    }

    let card_entity: HashMap<CardId, Entity> = bf_cards.iter().map(|(e, gid)| (gid.0, e)).collect();

    // Each card's current piles, read off its coins: (kind, coins).
    let mut existing_piles: HashMap<Entity, Vec<(CounterType, u32)>> = HashMap::new();
    let mut existing_entities: HashMap<Entity, Vec<Entity>> = HashMap::new();
    {
        let mut accum: HashMap<(Entity, CounterType), u32> = HashMap::new();
        for (coin_e, child_of, coin) in &existing {
            *accum.entry((child_of.parent(), coin.kind)).or_default() += 1;
            existing_entities.entry(child_of.parent()).or_default().push(coin_e);
        }
        for ((parent, kind), n) in accum {
            existing_piles.entry(parent).or_default().push((kind, n));
        }
        for piles in existing_piles.values_mut() {
            piles.sort_by_key(|(k, _)| sort_key(*k));
        }
    }

    let mut on_battlefield: HashSet<Entity> = HashSet::new();
    for p in &cv.battlefield {
        let Some(&parent) = card_entity.get(&p.id) else { continue };
        on_battlefield.insert(parent);

        let desired: Vec<(CounterType, u32)> = board_counters(p)
            .into_iter()
            .map(|(k, n)| (k, pile_height(n)))
            .collect();
        if existing_piles.get(&parent).map(Vec::as_slice).unwrap_or(&[]) == desired.as_slice() {
            continue;
        }
        if let Some(coins) = existing_entities.remove(&parent) {
            for e in coins {
                commands.entity(e).despawn();
            }
        }

        // The chip is turned about its Y; this lays that axis along the
        // card's +Z, so the chip lies flat on the card face up.
        let lay_flat = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
        let had = |kind: CounterType| {
            existing_piles.get(&parent).and_then(|piles| piles.iter().find(|(k, _)| *k == kind)).map_or(0, |&(_, n)| n)
        };
        for (slot, &(kind, coins)) in desired.iter().enumerate() {
            let chip = assets
                .chips
                .entry(kind)
                .or_insert_with(|| meshes.add(super::coin_mesh::chip_mesh(COIN_RADIUS, COIN_HEIGHT, coin_body(kind))))
                .clone();
            let top = pile_top(slot, coins);
            let had = had(kind);
            commands.entity(parent).with_children(|card| {
                for i in 0..coins {
                    // Each coin turned a little from the one under it, so a
                    // pile's spots don't line up into stripes, and set a hair
                    // off-centre, as a hand stacks them.
                    let twist = Quat::from_rotation_y(i as f32 * 0.9);
                    let off = if i == 0 { Vec2::ZERO } else { Vec2::from_angle(i as f32 * 2.4) * 0.012 };
                    let rest = Vec3::new(top.x + off.x, top.y + off.y, COIN_BASE_Z + (i as f32 + 0.5) * COIN_HEIGHT);
                    let mut coin = card.spawn((
                        Mesh3d(chip.clone()),
                        MeshMaterial3d(assets.material.clone()),
                        Transform::from_translation(rest).with_rotation(lay_flat * twist),
                        CounterCoin { card_id: p.id, kind },
                    ));
                    // A coin the pile didn't have falls onto it.
                    if i >= had {
                        let elapsed = -((i - had) as f32) * DROP_STAGGER;
                        coin.insert((
                            CoinDrop { rest_z: rest.z, elapsed },
                            Transform::from_translation(rest + Vec3::Z * drop_lift(elapsed))
                                .with_rotation(lay_flat * twist),
                        ));
                    }
                }
            });
        }
    }

    // Coins whose card has left the battlefield.
    for (parent, coins) in existing_entities {
        if !on_battlefield.contains(&parent) {
            for e in coins {
                commands.entity(e).despawn();
            }
        }
    }
}

// ── Counter labels ────────────────────────────────────────────────────────────
//
// The coin says "some of this colour"; the label says what and how many: the
// count centred on the pile's top coin, the kind's name in a small tag beside
// it, on the side facing into the card.

/// Below default-z UI so peek popups / tooltips / modals draw over it.
const COUNTER_LABEL_Z: i32 = crate::theme::layer::CARD_OVERLAY;

/// Short, human-readable token naming a counter kind for the count label.
fn counter_token(kind: CounterType) -> &'static str {
    match kind {
        CounterType::PlusOnePlusOne => "+1/+1",
        CounterType::MinusOneMinusOne => "-1/-1",
        CounterType::MinusZeroMinusOne => "-0/-1",
        CounterType::MinusOneMinusZero => "-1/-0",
        CounterType::PlusOnePlusZero => "+1/+0",
        CounterType::PlusZeroPlusOne => "+0/+1",
        CounterType::PlusTwoPlusZero => "+2/+0",
        CounterType::PlusZeroPlusTwo => "+0/+2",
        CounterType::PlusTwoPlusTwo => "+2/+2",
        CounterType::Currency => "Currency",
        CounterType::MinusZeroMinusTwo => "-0/-2",
        CounterType::Glyph => "Glyph",
        CounterType::Sleep => "Sleep",
        CounterType::Matrix => "Matrix",
        CounterType::Hatchling => "Hatchling",
        CounterType::Intervention => "Intervention",
        CounterType::Scream => "Scream",
        CounterType::Doom => "Doom",
        CounterType::Pupa => "Pupa",
        CounterType::Dream => "Dream",
        CounterType::Pin => "Pin",
        CounterType::Loyalty => "Loyalty",
        CounterType::Charge => "Charge",
        CounterType::Manifestation => "Manifestation",
        CounterType::Mine => "Mine",
        CounterType::Delay => "Delay",
        CounterType::Shred => "Shred",
        CounterType::Carrion => "Carrion",
        CounterType::Stun => "Stun",
        CounterType::Time => "Time",
        CounterType::Poison => "Poison",
        CounterType::Energy => "Energy",
        CounterType::Lore => "Lore",
        CounterType::Fade => "Fade",
        CounterType::Blood => "Blood",
        CounterType::Plague => "Plague",
        CounterType::Fuse => "Fuse",
        CounterType::Omen => "Omen",
        CounterType::Level => "Level",
        CounterType::Experience => "XP",
        CounterType::Shield => "Shield",
        CounterType::Ki => "Ki",
        CounterType::Tide => "Tide",
        CounterType::Flood => "Flood",
        CounterType::Bounty => "Bounty",
        CounterType::Valor => "Valor",
        CounterType::Oil => "Oil",
        CounterType::Blight => "Blight",
        CounterType::Invitation => "Invitation",
        CounterType::Finality => "Finality",
        CounterType::Indestructible => "Indestructible",
        CounterType::Quest => "Quest",
        CounterType::Verse => "Verse",
        CounterType::Impostor => "Impostor",
        CounterType::Bloodstain => "Bloodstain",
        CounterType::Page => "Page",
        CounterType::Plot => "Plot",
        CounterType::Growth => "Growth",
        CounterType::Ice => "Ice",
        CounterType::Soot => "Soot",
        CounterType::Fate => "Fate",
        CounterType::Void => "Void",
        CounterType::Coin => "Coin",
        CounterType::Wish => "Wish",
        CounterType::Study => "Study",
        CounterType::Hone => "Hone",
        CounterType::Burden => "Burden",
        CounterType::Luck => "Luck",
        CounterType::Age => "Age",
        CounterType::Fire => "Fire",
        CounterType::Conqueror => "Conqueror",
        CounterType::Muster => "Muster",
        CounterType::Acorn => "Acorn",
        CounterType::Defense => "Defense",
        CounterType::Nest => "Nest",
        CounterType::Possession => "Possession",
        CounterType::Incubation => "Incubation",
        CounterType::Revival => "Revival",
        CounterType::Stash => "Stash",
        CounterType::Rev => "Rev",
        CounterType::Divinity => "Divinity",
        CounterType::Devotion => "Devotion",
        CounterType::Aim => "Aim",
        CounterType::Theft => "Theft",
        CounterType::Training => "Training",
        CounterType::Fellowship => "Fellowship",
        CounterType::Bait => "Bait",
        CounterType::Supply => "Supply",
        CounterType::Book => "Book",
        CounterType::Point => "Point",
        CounterType::Unlock => "Unlock",
        CounterType::Silver => "Silver",
        CounterType::Prepared => "Prepared",
        CounterType::Palliation => "Palliation",
        CounterType::Eon => "Eon",
        CounterType::Blaze => "Blaze",
        CounterType::Phylactery => "Phylactery",
        CounterType::Filibuster => "Filibuster",
        CounterType::Petal => "Petal",
        CounterType::Arrow => "Arrow",
        CounterType::Infection => "Infection",
        CounterType::Fungus => "Fungus",
        CounterType::Storage => "Storage",
        CounterType::Depletion => "Depletion",
        CounterType::Shell => "Shell",
        CounterType::Hourglass => "Hourglass",
        CounterType::Feather => "Feather",
        CounterType::Gold => "Gold",
        CounterType::Trap => "Trap",
        CounterType::Polyp => "Polyp",
        CounterType::Ore => "Ore",
        CounterType::Death => "Death",
        CounterType::Rust => "Rust",
        CounterType::Corruption => "Corruption",
        CounterType::Bloodline => "Bloodline",
        CounterType::Slime => "Slime",
        CounterType::Descent => "Descent",
        CounterType::Corpse => "Corpse",
        CounterType::Rally => "Rally",
        CounterType::Enlightened => "Enlightened",
        CounterType::Spite => "Spite",
        CounterType::Strife => "Strife",
        CounterType::Fury => "Fury",
        CounterType::Night => "Night",
        CounterType::Bribery => "Bribery",
        CounterType::Hoofprint => "Hoofprint",
        CounterType::Soul => "Soul",
        CounterType::Voyage => "Voyage",
        CounterType::Ritual => "Ritual",
        CounterType::Cage => "Cage",
        CounterType::Blessing => "Blessing",
        CounterType::Component => "Component",
        CounterType::Ingredient => "Ingredient",
        CounterType::Landmark => "Landmark",
        CounterType::Plan => "Plan",
        CounterType::Takeover => "Takeover",
        CounterType::Necrodermis => "Necrodermis",
        CounterType::Vow => "Vow",
        CounterType::Duty => "Duty",
        CounterType::Slumber => "Slumber",
        CounterType::Story => "Story",
        CounterType::Winch => "Winch",
        CounterType::Wind => "Wind",
        CounterType::Pressure => "Pressure",
        CounterType::Hunger => "Hunger",
        CounterType::Elixir => "Elixir",
        CounterType::Pain => "Pain",
        CounterType::Magnet => "Magnet",
        CounterType::Suspect => "Suspect",
        CounterType::Hit => "Hit",
        CounterType::Everything => "Everything",
        CounterType::Contested => "Contested",
        CounterType::Eyeball => "Eyeball",
        CounterType::Brick => "Brick",
        CounterType::Cell => "Cell",
        CounterType::Unity => "Unity",
        CounterType::Ribbon => "Ribbon",
        CounterType::Brain => "Brain",
    }
}

/// Bright, legible text colour for each counter kind's tag; the coin fill is
/// its deep shade ([`coin_body`]).
fn counter_label_color(kind: CounterType) -> Color {
    match kind {
        CounterType::PlusOnePlusOne
        | CounterType::PlusOnePlusZero
        | CounterType::PlusZeroPlusOne
        | CounterType::PlusTwoPlusZero
        | CounterType::PlusZeroPlusTwo
        | CounterType::PlusTwoPlusTwo => Color::srgb(0.45, 0.95, 0.50),
        CounterType::MinusOneMinusOne
        | CounterType::MinusZeroMinusOne
        | CounterType::MinusOneMinusZero
        | CounterType::MinusZeroMinusTwo => Color::srgb(0.98, 0.48, 0.48),
        CounterType::Loyalty => Color::srgb(0.96, 0.82, 0.32),
        CounterType::Charge => Color::srgb(0.42, 0.85, 0.96),
        CounterType::Stun => Color::srgb(0.82, 0.56, 0.96),
        CounterType::Time => Color::srgb(0.96, 0.74, 0.42),
        CounterType::Lore => Color::srgb(0.92, 0.66, 0.84),
        CounterType::Poison => Color::srgb(0.56, 0.86, 0.42),
        CounterType::Energy => Color::srgb(0.46, 0.66, 0.98),
        CounterType::Ki => Color::srgb(0.96, 0.62, 0.36),
        CounterType::Tide => Color::srgb(0.40, 0.74, 0.96),
        CounterType::Flood => Color::srgb(0.30, 0.62, 0.88),
        CounterType::Bounty => Color::srgb(0.96, 0.66, 0.36),
        CounterType::Fire => Color::srgb(0.98, 0.44, 0.24),
        CounterType::Conqueror => Color::srgb(0.90, 0.36, 0.30),
        _ => Color::srgb(0.86, 0.86, 0.92),
    }
}

/// The tag naming a pile: the counter's name, or "Impending" for the time
/// counters an Impending permanent counts down with (CR 702.183).
fn counter_tag(kind: CounterType, impending: bool) -> &'static str {
    if impending && kind == CounterType::Time { "Impending" } else { counter_token(kind) }
}

/// Which side of its coin a tag hangs on: toward the card's local +X (into
/// the art box) as that direction falls on screen, so an opponent's card,
/// which faces them, and a tapped card keep their tags on the card.
#[derive(Clone, Copy, PartialEq, Debug)]
enum TagSide {
    Right,
    Left,
    Below,
    Above,
}

fn tag_side(into_card: Vec2) -> TagSide {
    if into_card.x.abs() >= into_card.y.abs() {
        if into_card.x >= 0.0 { TagSide::Right } else { TagSide::Left }
    } else if into_card.y >= 0.0 {
        TagSide::Below
    } else {
        TagSide::Above
    }
}

/// Place a tag on `side` of its coin's box: flush against it, centred on
/// the other axis.
fn place_tag(node: &mut Node, transform: &mut UiTransform, side: TagSide) {
    let gap = Val::Px(2.0);
    node.left = Val::Auto;
    node.right = Val::Auto;
    node.top = Val::Auto;
    node.bottom = Val::Auto;
    node.margin = UiRect::ZERO;
    match side {
        TagSide::Right => {
            node.left = Val::Percent(100.0);
            node.top = Val::Percent(50.0);
            node.margin.left = gap;
            transform.translation = Val2::percent(0.0, -50.0);
        }
        TagSide::Left => {
            node.right = Val::Percent(100.0);
            node.top = Val::Percent(50.0);
            node.margin.right = gap;
            transform.translation = Val2::percent(0.0, -50.0);
        }
        TagSide::Below => {
            node.top = Val::Percent(100.0);
            node.left = Val::Percent(50.0);
            node.margin.top = gap;
            transform.translation = Val2::percent(-50.0, 0.0);
        }
        TagSide::Above => {
            node.bottom = Val::Percent(100.0);
            node.left = Val::Percent(50.0);
            node.margin.bottom = gap;
            transform.translation = Val2::percent(-50.0, 0.0);
        }
    }
}

/// The count's font size for a coin `diameter` UI px across: two digits
/// inside the chip's face, which is 60 % of it — printed on the chip, it
/// grows with it seen up close (the Ctrl zoom).
fn count_font_size(diameter: f32) -> f32 {
    (diameter * 0.56).round().clamp(10.0, 48.0)
}

/// The tag's font size beside a coin `diameter` UI px across: a label, so
/// it grows more slowly than the count.
fn tag_font_size(diameter: f32) -> f32 {
    (diameter * 0.22).round().clamp(11.0, 20.0)
}

/// A pile's label, tied to a battlefield card's counter of one kind: a box
/// the size of the pile's top coin with the count centred in it
/// ([`CounterCount`]) and the kind's tag hung beside it ([`CounterTag`]).
#[derive(Component)]
pub struct CounterLabel {
    pub card_id: CardId,
    pub kind: CounterType,
}

#[derive(Component)]
pub struct CounterCount;

#[derive(Component)]
pub struct CounterTag(TagSide);

/// Where a pile's label goes this frame, in UI px: the top coin's centre,
/// its radius, and the screen direction into the card.
fn pile_on_screen(
    camera: &Camera,
    cam_xform: &GlobalTransform,
    ui_scale: &UiScale,
    card: &GlobalTransform,
    slot: usize,
    count: u32,
) -> Option<(Vec2, f32, Vec2)> {
    let project = |local: Vec3| {
        crate::theme::project_to_ui(camera, cam_xform, ui_scale, card.transform_point(local))
    };
    let top = pile_top(slot, count);
    let centre = project(top)?;
    let along_x = project(top + Vec3::X * COIN_RADIUS)? - centre;
    let along_y = project(top + Vec3::Y * COIN_RADIUS)? - centre;
    Some((centre, along_x.length().max(along_y.length()), along_x))
}

/// Reconcile the pile labels with the engine view, and put each on its
/// pile's top coin.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn sync_counter_labels(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<crate::theme::UiFonts>,
    cards: Query<(Entity, &GameCardId, &GlobalTransform), With<BattlefieldCard>>,
    cover_cards: crate::card::cover::CoverQuery,
    camera_q: Query<(&Camera, &GlobalTransform), With<crate::MainCamera>>,
    ui_scale: Res<UiScale>,
    mut labels: Query<(Entity, &CounterLabel, &mut Node, &Children), Without<CounterTag>>,
    mut counts: Query<(&mut Text, &mut TextFont), (With<CounterCount>, Without<CounterTag>)>,
    mut tags: Query<
        (&mut Text, &mut TextFont, &mut CounterTag, &mut Node, &mut UiTransform),
        Without<CounterCount>,
    >,
    mut coins: Query<(&CounterCoin, &mut Visibility)>,
    mut desired_cache: Local<HashMap<(CardId, CounterType), (u32, usize, bool)>>,
) {
    let Some(cv) = &view.0 else {
        for (e, ..) in &mut labels {
            commands.entity(e).despawn();
        }
        return;
    };
    let Ok((camera, cam_xform)) = camera_q.single() else { return };
    let card_xform: HashMap<CardId, (Entity, &GlobalTransform)> =
        cards.iter().map(|(e, g, t)| (g.0, (e, t))).collect();
    // A pile hides, coins and label, while another card lies over the patch
    // of card under it (`card::cover`). Not over the pile's top: the card on
    // top of a wrapped row sits a few hundredths higher, and a pile poked
    // up through it — a counter on the card underneath, seemingly on it.
    let cover = crate::card::cover::CardCover::new(cam_xform, &cover_cards);
    let mut covered: HashSet<(CardId, CounterType)> = HashSet::new();

    // (card, kind) → (count, column slot, impending). Rebuilt on a view
    // change; positions track every frame.
    if view.is_changed() {
        desired_cache.clear();
        for p in &cv.battlefield {
            let impending = p.impending_counters.unwrap_or(0) > 0;
            for (slot, (k, n)) in board_counters(p).into_iter().enumerate() {
                desired_cache.insert((p.id, k), (n, slot, impending));
            }
        }
    }
    let desired = &*desired_cache;

    let mut seen: HashSet<(CardId, CounterType)> = HashSet::new();
    for (e, label, mut node, children) in &mut labels {
        let Some(&(count, slot, impending)) = desired.get(&(label.card_id, label.kind)) else {
            commands.entity(e).despawn();
            continue;
        };
        seen.insert((label.card_id, label.kind));
        let card = card_xform.get(&label.card_id);
        if card.is_some_and(|(e, t)| cover.hides_local(*e, t, pile_base(slot))) {
            covered.insert((label.card_id, label.kind));
        }
        let Some((centre, radius, into_card)) = card
            .filter(|_| !covered.contains(&(label.card_id, label.kind)))
            .and_then(|(_, t)| pile_on_screen(camera, cam_xform, &ui_scale, t, slot, count))
        else {
            node.display = Display::None;
            continue;
        };
        node.display = Display::Flex;
        node.left = Val::Px(centre.x);
        node.top = Val::Px(centre.y);
        node.width = Val::Px(radius * 2.0);
        node.height = Val::Px(radius * 2.0);
        for child in children.iter() {
            if let Ok((mut text, mut font)) = counts.get_mut(child) {
                let n = count.to_string();
                if text.0 != n {
                    text.0 = n;
                    commands.entity(e).try_insert(crate::theme::OverlayPulse::default());
                }
                let size = FontSize::Px(count_font_size(radius * 2.0));
                if font.font_size != size {
                    font.font_size = size;
                }
            } else if let Ok((mut text, mut font, mut tag, mut tag_node, mut transform)) = tags.get_mut(child) {
                let size = FontSize::Px(tag_font_size(radius * 2.0));
                if font.font_size != size {
                    font.font_size = size;
                }
                let name = counter_tag(label.kind, impending);
                if text.0 != name {
                    text.0 = name.to_string();
                }
                let side = tag_side(into_card);
                if tag.0 != side {
                    tag.0 = side;
                    place_tag(&mut tag_node, &mut transform, side);
                }
            }
        }
    }

    for (coin, mut visibility) in &mut coins {
        let want = if covered.contains(&(coin.card_id, coin.kind)) {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if *visibility != want {
            *visibility = want;
        }
    }

    for (&(id, kind), &(count, _, impending)) in desired.iter() {
        if seen.contains(&(id, kind)) || !card_xform.contains_key(&id) {
            continue;
        }
        let color = counter_label_color(kind);
        let mut tag_node = Node {
            position_type: PositionType::Absolute,
            padding: UiRect::axes(Val::Px(3.0), Val::Px(0.0)),
            border_radius: BorderRadius::all(Val::Px(3.0)),
            ..default()
        };
        let mut tag_transform = UiTransform::IDENTITY;
        place_tag(&mut tag_node, &mut tag_transform, TagSide::Right);
        // Parked off-screen until the next frame places it.
        commands
            .spawn((
                CounterLabel { card_id: id, kind },
                crate::systems::focus::CardOverlay(id),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(-1000.0),
                    top: Val::Px(-1000.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                // Centred on the coin: left/top are the coin's centre.
                UiTransform::from_translation(Val2::percent(-50.0, -50.0)),
                // A new pile swells in, as a changed count does.
                crate::theme::OverlayPulse::default(),
                Pickable::IGNORE,
                GlobalZIndex(COUNTER_LABEL_Z),
                crate::systems::game_ui::InGameRoot,
            ))
            .with_children(|label| {
                label.spawn((
                    CounterCount,
                    Text::new(count.to_string()),
                    ui_fonts.tf(14.0),
                    TextColor(count_ink(kind)),
                    // The UI font is a Light cut; a copy of the digits a
                    // hair to the right thickens them enough to read.
                    TextShadow { offset: Vec2::new(0.7, 0.0), color: count_ink(kind) },
                    Pickable::IGNORE,
                ));
                label.spawn((
                    CounterTag(TagSide::Right),
                    Text::new(counter_tag(kind, impending)),
                    ui_fonts.tf(11.0),
                    // Its box is the coin's: "+1/+1" wrapped in half.
                    TextLayout::no_wrap(),
                    TextColor(color),
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.78)),
                    tag_node,
                    tag_transform,
                    Pickable::IGNORE,
                ));
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_gameplay_counters_match_the_tooltip_names() {
        // Book / Point / Unlock are real gameplay counters (Cases, Cryptex,
        // etc.); the board coin must name them, not fall through to "Counter".
        assert_eq!(counter_token(CounterType::Book), "Book");
        assert_eq!(counter_token(CounterType::Point), "Point");
        assert_eq!(counter_token(CounterType::Unlock), "Unlock");
    }

    #[test]
    fn a_pile_is_capped_and_its_tag_names_the_kind() {
        assert_eq!(pile_height(1), 1);
        assert_eq!(pile_height(3), 3);
        assert_eq!(pile_height(11), MAX_PILE);
        assert_eq!(counter_tag(CounterType::Unlock, false), "Unlock");
        // Impending Time badges the countdown instead of a plain "Time".
        assert_eq!(counter_tag(CounterType::Time, true), "Impending");
        assert_eq!(counter_tag(CounterType::Time, false), "Time");
    }

    #[test]
    fn a_tag_hangs_toward_the_card() {
        assert_eq!(tag_side(Vec2::new(12.0, 1.0)), TagSide::Right);
        // An opponent's card faces them: its +X points left on screen.
        assert_eq!(tag_side(Vec2::new(-12.0, -1.0)), TagSide::Left);
        // A tapped card's +X runs up or down the screen.
        assert_eq!(tag_side(Vec2::new(1.0, 9.0)), TagSide::Below);
        assert_eq!(tag_side(Vec2::new(-1.0, -9.0)), TagSide::Above);
    }

    #[test]
    fn a_counter_another_badge_reads_gets_no_coin() {
        let mut p = crate::systems::counter_tooltip::tests::make_permanent_view(0, 1);
        p.card_types = vec![CardType::Planeswalker];
        p.counters = vec![(CounterType::Loyalty, 5), (CounterType::Charge, 1)];
        assert_eq!(board_counters(&p), vec![(CounterType::Charge, 1)]);
        // A creature's +1/+1 counters lead, its stun is the status chip's.
        p.card_types = vec![CardType::Creature];
        p.counters = vec![
            (CounterType::Stun, 1),
            (CounterType::Lore, 2),
            (CounterType::PlusOnePlusOne, 3),
            (CounterType::Charge, 0),
        ];
        assert_eq!(board_counters(&p), vec![(CounterType::PlusOnePlusOne, 3), (CounterType::Lore, 2)]);
        // A stunned land has no status chip, so its stun is a coin.
        p.card_types = vec![CardType::Land];
        p.counters = vec![(CounterType::Lore, 2), (CounterType::Stun, 1)];
        assert_eq!(board_counters(&p), vec![(CounterType::Stun, 1), (CounterType::Lore, 2)]);
    }
}
