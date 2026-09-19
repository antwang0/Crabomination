//! Commander (CR 903) table affordances.
//!
//! - **Command-zone cost badge** — a screen-space chip under each commander
//!   in a command zone giving its full next-cast cost, commander tax included
//!   (CR 903.8: {2} more per earlier cast from the command zone, tracked per
//!   commander so partners each show their own): `{4}{G} (+2 tax)`.
//! - **Castable border** — the viewer's command-zone commander gets the same
//!   green "castable now" ring a playable hand card wears, driven by the
//!   server's `ClientView::castable_command` dry-run.
//! - **Hover lines** — [`commander_hover_lines`] names the commander damage a
//!   hovered commander has dealt each other player (`Alice 12/21`); the hover
//!   preview (`systems::ui::hover_card_preview`) appends them.
//! - **Lethal warning** — a red `☠ 21` chip over an unblocked attacking
//!   commander whose hit would take its defender to 21 (CR 903.10a), plus a
//!   banner when the defender is the viewer.
//! - **Chip pulse** — the HUD's `⚔ name N/21` chip whose tally just rose
//!   pulses for a couple of seconds so the hit reads.
//!
//! Badges mirror `free_cast_badge`: screen-space nodes reprojected from the
//! card's world position and reconciled against the view every frame.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use crabomination::card::CardId;
use crabomination::game::TurnStep;
use crabomination::mana::{ManaCost, ManaSymbol};
use crabomination::net::{ClientView, CommanderDamageEntry, HandCardView, PlayerView};

use crate::MainCamera;
use crate::card::{
    BattlefieldCard, CARD_HEIGHT, CARD_THICKNESS, CardHighlightAssets, CommandZoneCard,
    GameCardId,
};
use crate::net_plugin::CurrentView;
use crate::systems::game_ui::InGameRoot;
use crate::theme::{self, UiFonts};

/// CR 903.10a — combat damage from a single commander that loses the game.
pub const COMMANDER_DAMAGE_LETHAL: u32 = 21;

/// How long the HUD commander-damage chip pulses after its tally rises.
const CHIP_FLASH_SECS: f32 = 2.0;

const BADGE_Z: i32 = theme::layer::CARD_OVERLAY;

// ── Pure helpers ────────────────────────────────────────────────────────────

/// CR 903.8 — the commander tax for the next cast: {2} per earlier cast from
/// the command zone.
pub fn commander_tax(casts: u32) -> u32 {
    2 * casts
}

/// `cost` with the commander tax folded into its generic pip, so `{2}{G}`
/// after one cast reads `{4}{G}` rather than `{2}{G}{2}`. X stays first and
/// every non-generic pip keeps its printed order.
pub fn taxed_cost(cost: &ManaCost, casts: u32) -> ManaCost {
    let generic = cost.generic_total() + commander_tax(casts);
    let xs = cost.symbols.iter().filter(|s| matches!(s, ManaSymbol::X)).copied();
    let rest = cost
        .symbols
        .iter()
        .filter(|s| !matches!(s, ManaSymbol::X | ManaSymbol::Generic(_)))
        .copied();
    let mut symbols: Vec<ManaSymbol> = xs.collect();
    if generic > 0 {
        symbols.push(ManaSymbol::Generic(generic));
    }
    symbols.extend(rest);
    ManaCost { symbols }
}

/// The command-zone badge text: the full next-cast cost, plus the tax share
/// once there is one — `{2}{G}` fresh, `{4}{G} (+2 tax)` after one cast.
pub fn command_zone_cost_label(cost: &ManaCost, casts: u32) -> String {
    let full = taxed_cost(cost, casts).summary();
    match commander_tax(casts) {
        0 => full,
        tax => format!("{full} (+{tax} tax)"),
    }
}

/// How many times `player` has cast the commander named `name` from the
/// command zone (`PlayerView::commander_casts` is keyed by name, one entry per
/// commander, so partners never share a count).
pub fn commander_cast_count(player: &PlayerView, name: &str) -> u32 {
    player
        .commander_casts
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, c)| *c)
        .unwrap_or(0)
}

/// The seat whose commander `id` is, if it is anyone's.
pub fn commander_owner(cv: &ClientView, id: CardId) -> Option<usize> {
    cv.players.iter().find(|p| p.commanders.contains(&id)).map(|p| p.seat)
}

/// Does this tally entry belong to commander `id` (named `name`)? Matches on
/// the source id; an entry from a snapshot predating `source_id` falls back
/// to the name.
fn entry_is_from(entry: &CommanderDamageEntry, id: CardId, name: &str) -> bool {
    match entry.source_id {
        Some(src) => src == id,
        None => entry.source_name == name,
    }
}

/// `(player name, damage)` — the combat damage commander `id` has dealt each
/// other live player (CR 903.10a), in seat order, zeros included so the pod
/// reads at a glance. Empty when `id` isn't a commander.
pub fn commander_damage_dealt(cv: &ClientView, id: CardId, name: &str) -> Vec<(String, u32)> {
    let Some(owner) = commander_owner(cv, id) else { return Vec::new() };
    cv.players
        .iter()
        .filter(|p| p.seat != owner && !p.eliminated)
        .map(|p| {
            let amount = p
                .commander_damage_taken
                .iter()
                .find(|e| entry_is_from(e, id, name))
                .map(|e| e.amount)
                .unwrap_or(0);
            (p.name.clone(), amount)
        })
        .collect()
}

/// Hover-preview lines for commander `id`: a header plus one `Name N/21` row
/// per other player. Empty for a non-commander. The bool is the preview's
/// "reminder" styling flag (secondary colour) — the header is primary.
pub fn commander_hover_lines(cv: &ClientView, id: CardId, name: &str) -> Vec<(String, bool)> {
    let dealt = commander_damage_dealt(cv, id, name);
    if dealt.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![("\u{2694} Commander damage dealt:".to_string(), false)];
    lines.extend(dealt.into_iter().map(|(who, n)| {
        let lethal = if n >= COMMANDER_DAMAGE_LETHAL { "  \u{2620}" } else { "" };
        (format!("  {who} {n}/{COMMANDER_DAMAGE_LETHAL}{lethal}"), true)
    }));
    lines
}

/// One attacking commander whose unblocked hit is lethal commander damage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LethalCommanderThreat {
    pub attacker: CardId,
    pub attacker_name: String,
    pub defender: usize,
    /// The defender's tally from this commander after the hit.
    pub total: u32,
}

/// CR 903.10a — every attacking commander that is (so far) unblocked and
/// whose power, added to what it has already dealt the player it attacks,
/// reaches 21. Only attacks on a *player* count: damage to a planeswalker or
/// battle isn't commander damage to anyone. Blockers the view already shows
/// clear the warning; power ≤ 0 never threatens.
pub fn lethal_commander_threats(cv: &ClientView) -> Vec<LethalCommanderThreat> {
    let blocked: HashSet<CardId> = cv
        .battlefield
        .iter()
        .flat_map(|p| p.blocking_attackers.iter().copied())
        .collect();
    cv.battlefield
        .iter()
        .filter(|p| p.attacking && p.power > 0 && !blocked.contains(&p.id))
        .filter(|p| commander_owner(cv, p.id).is_some())
        .filter_map(|p| {
            // Commander damage counts against a player hit directly, not a
            // planeswalker's controller (CR 903.10a).
            let crabomination::game::AttackTarget::Player(defender) = p.attack_target? else {
                return None;
            };
            let dp = cv.players.iter().find(|d| d.seat == defender)?;
            let already = dp
                .commander_damage_taken
                .iter()
                .find(|e| entry_is_from(e, p.id, &p.name))
                .map(|e| e.amount)
                .unwrap_or(0);
            let total = already + p.power as u32;
            (total >= COMMANDER_DAMAGE_LETHAL).then(|| LethalCommanderThreat {
                attacker: p.id,
                attacker_name: p.name.clone(),
                defender,
                total,
            })
        })
        .collect()
}

/// Identity of one HUD commander-damage chip's source: id when the view
/// carries it, else the name.
pub type ChipSource = (Option<CardId>, String);

fn chip_source(entry: &CommanderDamageEntry) -> ChipSource {
    (entry.source_id, entry.source_name.clone())
}

/// Every `(victim, source) → amount` tally in the view.
pub fn commander_damage_tallies(cv: &ClientView) -> HashMap<(usize, ChipSource), u32> {
    cv.players
        .iter()
        .flat_map(|p| p.commander_damage_taken.iter().map(move |e| ((p.seat, chip_source(e)), e.amount)))
        .collect()
}

/// `(source, new amount)` for every tally that rose between `prev` and `now`
/// — a first hit (absent before) counts as a rise from zero.
pub fn risen_tallies(
    prev: &HashMap<(usize, ChipSource), u32>,
    now: &HashMap<(usize, ChipSource), u32>,
) -> Vec<(ChipSource, u32)> {
    let mut out: Vec<(ChipSource, u32)> = now
        .iter()
        .filter(|(k, v)| prev.get(*k).copied().unwrap_or(0) < **v)
        .map(|((_, src), v)| (src.clone(), *v))
        .collect();
    out.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.1.cmp(&b.0.1)));
    out
}

// ── Command-zone cost badge + castable border ───────────────────────────────

/// Screen-space cost chip under a command-zone commander.
#[derive(Component)]
pub struct CommandZoneCostBadge(pub CardId);

/// Every commander in every command zone with its badge label. Conspiracies
/// and Vanguards sharing the zone aren't commanders and get none.
fn command_zone_cost_labels(cv: &ClientView) -> HashMap<CardId, String> {
    let mut out = HashMap::new();
    for p in &cv.players {
        for entry in &p.command {
            if let HandCardView::Known(k) = entry
                && p.commanders.contains(&k.id)
            {
                let casts = commander_cast_count(p, &k.name);
                out.insert(k.id, command_zone_cost_label(&k.cost, casts));
            }
        }
    }
    out
}

/// Reconcile the command-zone cost chips with the view. Runs every frame.
#[allow(clippy::type_complexity)]
pub fn sync_command_zone_cost_badges(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    cards: Query<(&GameCardId, &GlobalTransform), With<CommandZoneCard>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut badges: Query<(Entity, &CommandZoneCostBadge, &mut Node, &mut Text)>,
) {
    let desired = view.0.as_ref().map(command_zone_cost_labels).unwrap_or_default();
    let Ok((camera, cam_xform)) = camera_q.single() else { return };

    // Anchor under the card's bottom edge.
    let bottom_local = Vec3::new(0.0, -CARD_HEIGHT / 2.0, 0.0);
    let anchor_of: HashMap<CardId, Vec2> = cards
        .iter()
        .filter(|(gid, _)| desired.contains_key(&gid.0))
        .filter_map(|(gid, gtf)| {
            camera
                .world_to_viewport(cam_xform, gtf.transform_point(bottom_local))
                .ok()
                .map(|v| (gid.0, v))
        })
        .collect();
    // Rough centring: the node auto-sizes, so offset by an estimated width.
    let place = |label: &str, at: Vec2| (at.x - label.chars().count() as f32 * 3.4, at.y + 2.0);

    let mut seen: HashSet<CardId> = HashSet::new();
    for (e, badge, mut node, mut text) in &mut badges {
        let Some(label) = desired.get(&badge.0) else {
            commands.entity(e).despawn();
            continue;
        };
        seen.insert(badge.0);
        if text.0 != *label {
            text.0 = label.clone();
        }
        match anchor_of.get(&badge.0) {
            Some(at) => {
                let (x, y) = place(label, *at);
                node.display = Display::Flex;
                node.left = Val::Px(x);
                node.top = Val::Px(y);
            }
            None => node.display = Display::None,
        }
    }
    for (id, label) in &desired {
        if seen.contains(id) {
            continue;
        }
        let (left, top) =
            anchor_of.get(id).map(|at| place(label, *at)).unwrap_or((-1000.0, -1000.0));
        commands.spawn((
            CommandZoneCostBadge(*id),
            Text::new(label.clone()),
            ui_fonts.tf(12.0),
            TextColor(theme::ACCENT_GOLD),
            BackgroundColor(Color::srgba(0.10, 0.09, 0.05, 0.92)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                padding: UiRect::axes(Val::Px(5.0), Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(BADGE_Z),
            InGameRoot,
        ));
    }
}

/// The green "castable now" ring on a command-zone commander. Its own
/// component rather than `CastableHighlight`, whose system strips the ring
/// from every non-hand card.
#[derive(Component)]
pub struct CommandZoneCastableHighlight {
    back: Entity,
    front: Entity,
}

/// Ring the viewer's castable command-zone commanders (`castable_command`),
/// matching the hand's green castable border. Off while targeting.
#[allow(clippy::type_complexity)]
pub fn update_command_zone_castable_highlights(
    mut commands: Commands,
    view: Res<CurrentView>,
    targeting: Res<crate::game::TargetingState>,
    highlight_assets: Option<Res<CardHighlightAssets>>,
    cards: Query<(Entity, &GameCardId, Option<&CommandZoneCastableHighlight>), With<CommandZoneCard>>,
) {
    let Some(assets) = highlight_assets else { return };
    let castable: HashSet<CardId> = match view.0.as_ref() {
        Some(cv) if !targeting.active => cv.castable_command.iter().copied().collect(),
        _ => HashSet::new(),
    };
    for (entity, gid, marker) in &cards {
        match (castable.contains(&gid.0), marker) {
            (true, None) => {
                let offset = CARD_THICKNESS / 2.0 + 0.0018;
                let mut ring = |z: f32| {
                    commands
                        .spawn((
                            Mesh3d(assets.border_mesh.clone()),
                            MeshMaterial3d(assets.castable_material.clone()),
                            Transform::from_xyz(0.0, 0.0, z),
                            Pickable::IGNORE,
                        ))
                        .id()
                };
                let (back, front) = (ring(-offset), ring(offset));
                commands
                    .entity(entity)
                    .insert(CommandZoneCastableHighlight { back, front })
                    .add_children(&[back, front]);
            }
            (false, Some(h)) => {
                commands.entity(h.back).despawn();
                commands.entity(h.front).despawn();
                commands.entity(entity).remove::<CommandZoneCastableHighlight>();
            }
            _ => {}
        }
    }
}

// ── Lethal commander-damage warning ─────────────────────────────────────────

/// Red `☠ N` chip over an attacking commander whose hit is lethal.
#[derive(Component)]
pub struct LethalCommanderChip(pub CardId);

/// Banner row shown to a defending viewer facing a lethal commander hit,
/// holding the text it was built with.
#[derive(Component)]
pub struct LethalCommanderBanner(String);

/// Is `step` a point in combat where an attacker's hit is still pending?
fn combat_pending(step: TurnStep) -> bool {
    matches!(
        step,
        TurnStep::DeclareAttackers | TurnStep::DeclareBlockers | TurnStep::FirstStrikeDamage
    )
}

/// The defending viewer's banner text for their lethal threats, if any.
pub fn lethal_banner_text(threats: &[LethalCommanderThreat], viewer: usize) -> Option<String> {
    let mine: Vec<String> = threats
        .iter()
        .filter(|t| t.defender == viewer)
        .map(|t| format!("{} ({}/{COMMANDER_DAMAGE_LETHAL})", t.attacker_name, t.total))
        .collect();
    (!mine.is_empty()).then(|| {
        format!("\u{2620} Lethal commander damage if unblocked: {}", mine.join(", "))
    })
}

/// Reconcile the lethal-threat chips and the viewer's banner. Runs every frame.
#[allow(clippy::type_complexity)]
pub fn sync_lethal_commander_warnings(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    cards: Query<(&GameCardId, &GlobalTransform), With<BattlefieldCard>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut chips: Query<(Entity, &LethalCommanderChip, &mut Node, &mut Text)>,
    banner: Query<(Entity, &LethalCommanderBanner)>,
) {
    let (threats, viewer) = match view.0.as_ref() {
        Some(cv) if combat_pending(cv.step) => (lethal_commander_threats(cv), cv.your_seat),
        Some(cv) => (Vec::new(), cv.your_seat),
        None => (Vec::new(), 0),
    };

    // Banner: rebuilt whenever its text changes (rare — once per combat).
    let text = lethal_banner_text(&threats, viewer);
    let current = banner.single().ok();
    if current.map(|(_, b)| Some(&b.0)) != Some(text.as_ref()) {
        if let Some((e, _)) = current {
            commands.entity(e).despawn();
        }
        if let Some(text) = text {
            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(96.0),
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    LethalCommanderBanner(text.clone()),
                    Pickable::IGNORE,
                    GlobalZIndex(BADGE_Z),
                    InGameRoot,
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new(text),
                        ui_fonts.tf(16.0),
                        TextColor(theme::TEXT_PRIMARY),
                        BackgroundColor(theme::HUD_BG_DANGER),
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(5.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                });
        }
    }

    // Per-attacker chips.
    let Ok((camera, cam_xform)) = camera_q.single() else { return };
    let want: HashMap<CardId, u32> = threats.iter().map(|t| (t.attacker, t.total)).collect();
    let top_local = Vec3::new(0.0, CARD_HEIGHT / 2.0, 0.0);
    let anchor_of: HashMap<CardId, Vec2> = cards
        .iter()
        .filter(|(gid, _)| want.contains_key(&gid.0))
        .filter_map(|(gid, gtf)| {
            camera
                .world_to_viewport(cam_xform, gtf.transform_point(top_local))
                .ok()
                .map(|v| (gid.0, Vec2::new(v.x - 18.0, v.y - 36.0)))
        })
        .collect();
    let label = |total: u32| format!("\u{2620} {total}");
    let mut seen = HashSet::new();
    for (e, chip, mut node, mut text) in &mut chips {
        let Some(total) = want.get(&chip.0) else {
            commands.entity(e).despawn();
            continue;
        };
        seen.insert(chip.0);
        let l = label(*total);
        if text.0 != l {
            text.0 = l;
        }
        match anchor_of.get(&chip.0) {
            Some(at) => {
                node.display = Display::Flex;
                node.left = Val::Px(at.x);
                node.top = Val::Px(at.y);
            }
            None => node.display = Display::None,
        }
    }
    for (id, total) in &want {
        if seen.contains(id) {
            continue;
        }
        let at = anchor_of.get(id).copied().unwrap_or(Vec2::splat(-1000.0));
        commands.spawn((
            LethalCommanderChip(*id),
            Text::new(label(*total)),
            ui_fonts.tf(14.0),
            TextColor(theme::TEXT_PRIMARY),
            BackgroundColor(Color::srgb(0.62, 0.08, 0.08)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(at.x),
                top: Val::Px(at.y),
                padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(BADGE_Z),
            InGameRoot,
        ));
    }
}

// ── HUD chip pulse ──────────────────────────────────────────────────────────

/// Tag on each HUD `⚔ name N/21` chip (spawned in `player_stats`), carrying
/// what the pulse needs to find it and restore it.
#[derive(Component)]
pub struct CommanderDamageChip {
    pub source: ChipSource,
    pub amount: u32,
    pub base_bg: Color,
}

impl CommanderDamageChip {
    pub fn new(entry: &CommanderDamageEntry, base_bg: Color) -> Self {
        Self { source: chip_source(entry), amount: entry.amount, base_bg }
    }
}

/// Last-seen tallies plus the chips currently pulsing. A pulse is keyed by
/// `(source, new amount)` rather than victim: the chip doesn't know its row,
/// and one commander landing the *same* new total on two players in one
/// view is the only way that over-flashes.
#[derive(Resource, Default)]
pub struct CommanderDamageFlash {
    primed: bool,
    last: HashMap<(usize, ChipSource), u32>,
    hot: Vec<(ChipSource, u32, f32)>,
}

/// Detect rising tallies on each view change and pulse the matching chips.
pub fn pulse_commander_damage_chips(
    view: Res<CurrentView>,
    time: Res<Time>,
    mut flash: ResMut<CommanderDamageFlash>,
    mut chips: Query<(&CommanderDamageChip, &mut BackgroundColor)>,
) {
    let Some(cv) = view.0.as_ref() else {
        // Between games: forget the tallies so the next game primes fresh.
        if flash.primed {
            *flash = CommanderDamageFlash::default();
        }
        return;
    };
    if view.is_changed() {
        let now = commander_damage_tallies(cv);
        if flash.primed {
            for (src, amount) in risen_tallies(&flash.last, &now) {
                flash.hot.retain(|(s, a, _)| !(s == &src && *a == amount));
                flash.hot.push((src, amount, CHIP_FLASH_SECS));
            }
        }
        flash.last = now;
        flash.primed = true;
    }
    let dt = time.delta_secs();
    for h in &mut flash.hot {
        h.2 -= dt;
    }
    flash.hot.retain(|h| h.2 > 0.0);

    let hot = Color::srgb(1.0, 0.30, 0.18);
    for (chip, mut bg) in &mut chips {
        let remaining = flash
            .hot
            .iter()
            .find(|(s, a, _)| *s == chip.source && *a == chip.amount)
            .map(|h| h.2);
        let want = match remaining {
            // Three quick pulses, easing out as the timer runs down.
            Some(r) => {
                let wave = 0.5 + 0.5 * (r * std::f32::consts::TAU * 1.5).cos();
                mix(chip.base_bg, hot, wave * (r / CHIP_FLASH_SECS).min(1.0).sqrt())
            }
            None => chip.base_bg,
        };
        if bg.0 != want {
            bg.0 = want;
        }
    }
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    let (a, b) = (a.to_srgba(), b.to_srgba());
    let l = |x: f32, y: f32| x + (y - x) * t.clamp(0.0, 1.0);
    Color::srgba(l(a.red, b.red), l(a.green, b.green), l(a.blue, b.blue), l(a.alpha, b.alpha))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crabomination::card::{CardDefinition, CardType, Supertype};
    use crabomination::format::Format;
    use crabomination::game::{Attack, AttackTarget, GameState, game_with_format};
    use crabomination::mana::{Color as ManaColor, cost, g, generic};
    use crabomination::server::view::project;

    fn commander(name: &'static str, power: i32) -> CardDefinition {
        CardDefinition {
            name,
            cost: cost(&[generic(2), g()]),
            supertypes: vec![Supertype::Legendary],
            card_types: vec![CardType::Creature],
            power,
            toughness: 3,
            ..Default::default()
        }
    }

    /// Four-seat Commander game; seat 0 fields `commander` on the battlefield
    /// (registered as its commander) attacking `defender`.
    fn attacking_commander(power: i32, defender: usize) -> (GameState, CardId) {
        let mut st = game_with_format(Format::Commander, 4);
        let id = st.add_card_to_battlefield(0, commander("Test Commander", power));
        st.players[0].commanders.push(id);
        st.attacking.push(Attack { attacker: id, target: AttackTarget::Player(defender) });
        st.step = TurnStep::DeclareBlockers;
        (st, id)
    }

    #[test]
    fn taxed_cost_folds_tax_into_the_generic_pip() {
        let c = cost(&[generic(2), g()]);
        assert_eq!(taxed_cost(&c, 0).summary(), "{2}{G}");
        assert_eq!(taxed_cost(&c, 1).summary(), "{4}{G}");
        // No printed generic: the tax becomes one.
        let green = cost(&[g(), g()]);
        assert_eq!(taxed_cost(&green, 2).summary(), "{4}{G}{G}");
        // X stays in front of the folded generic.
        let x = ManaCost { symbols: vec![ManaSymbol::Colored(ManaColor::Red), ManaSymbol::X] };
        assert_eq!(taxed_cost(&x, 1).summary(), "{X}{2}{R}");
    }

    #[test]
    fn command_zone_label_names_the_tax_only_once_there_is_one() {
        let c = cost(&[generic(2), g()]);
        assert_eq!(command_zone_cost_label(&c, 0), "{2}{G}");
        assert_eq!(command_zone_cost_label(&c, 1), "{4}{G} (+2 tax)");
        assert_eq!(command_zone_cost_label(&c, 3), "{8}{G} (+6 tax)");
    }

    /// Partners each carry their own tax (CR 903.8 counts per commander).
    #[test]
    fn partners_get_their_own_cost_labels() {
        let mut st = game_with_format(Format::Commander, 2);
        let ids = st.seat_commanders(0, vec![commander("Partner A", 2), commander("Partner B", 2)]);
        st.commander_cast_count.insert(ids[0], 2);
        let cv = project(&st, 0);
        let labels = command_zone_cost_labels(&cv);
        assert_eq!(labels[&ids[0]], "{6}{G} (+4 tax)");
        assert_eq!(labels[&ids[1]], "{2}{G}");
    }

    #[test]
    fn hover_lines_list_damage_dealt_to_each_other_player() {
        let (mut st, id) = attacking_commander(3, 1);
        st.commander_damage.insert((1, id), 12);
        st.commander_damage.insert((3, id), 21);
        let cv = project(&st, 0);
        let dealt = commander_damage_dealt(&cv, id, "Test Commander");
        let names: Vec<&str> = cv.players[1..].iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            dealt,
            vec![
                (names[0].to_string(), 12),
                (names[1].to_string(), 0),
                (names[2].to_string(), 21),
            ],
        );
        let lines = commander_hover_lines(&cv, id, "Test Commander");
        assert_eq!(lines.len(), 4, "header + three opponents: {lines:?}");
        assert!(lines[1].0.ends_with("12/21"), "{lines:?}");
        assert!(lines[3].0.contains("21/21") && lines[3].0.contains('\u{2620}'));
        // A plain permanent has no commander-damage lines.
        let bear = st.add_card_to_battlefield(0, crabomination::catalog::grizzly_bears());
        assert!(commander_hover_lines(&project(&st, 0), bear, "Grizzly Bears").is_empty());
    }

    #[test]
    fn lethal_threat_fires_only_when_the_hit_reaches_21_unblocked() {
        // 18 already + 3 power = 21 → lethal.
        let (mut st, id) = attacking_commander(3, 2);
        st.commander_damage.insert((2, id), 18);
        let threats = lethal_commander_threats(&project(&st, 2));
        assert_eq!(threats.len(), 1);
        assert_eq!((threats[0].attacker, threats[0].defender, threats[0].total), (id, 2, 21));
        assert!(lethal_banner_text(&threats, 2).is_some_and(|t| t.contains("21/21")));
        assert!(lethal_banner_text(&threats, 1).is_none(), "banner is the defender's");

        // 17 + 3 = 20 → not yet.
        st.commander_damage.insert((2, id), 17);
        assert!(lethal_commander_threats(&project(&st, 2)).is_empty());

        // A fresh 21-power commander is lethal on its own.
        let (st, _) = attacking_commander(21, 1);
        assert_eq!(lethal_commander_threats(&project(&st, 1)).len(), 1);
    }

    #[test]
    fn lethal_threat_ignores_non_commanders_and_non_player_attacks() {
        // A 25-power non-commander attacker is not commander damage.
        let (mut st, id) = attacking_commander(25, 1);
        st.players[0].commanders.clear();
        assert!(lethal_commander_threats(&project(&st, 1)).is_empty());
        // A commander attacking a planeswalker threatens no player.
        st.players[0].commanders.push(id);
        let pw = st.add_card_to_battlefield(1, crabomination::catalog::grizzly_bears());
        st.attacking = vec![Attack { attacker: id, target: AttackTarget::Planeswalker(pw) }];
        assert!(lethal_commander_threats(&project(&st, 1)).is_empty());
    }

    #[test]
    fn risen_tallies_reports_first_hits_and_increases_only() {
        let src = (Some(CardId(7)), "Cmdr".to_string());
        let mut prev = HashMap::new();
        prev.insert((1, src.clone()), 5);
        let mut now = prev.clone();
        assert!(risen_tallies(&prev, &now).is_empty());
        now.insert((1, src.clone()), 9);
        now.insert((2, src.clone()), 4);
        assert_eq!(risen_tallies(&prev, &now), vec![(src.clone(), 4), (src, 9)]);
    }
}
