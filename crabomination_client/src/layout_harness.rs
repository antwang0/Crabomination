//! Layout harness: boot a fixed busy board and screenshot it.
//!
//! `--layout-fixture <seats>` starts a local match from [`fixture_state`] — a
//! mid-game board with full hands, land and creature rows, graveyards and
//! exile, paused in the viewer's main phase — instead of a fresh deal, so two
//! builds can be compared on the same table. `--screenshot <path>` saves the
//! primary window once the view has been up for `--screenshot-delay` seconds
//! (default 8, for card art to stream in) and then exits. `--window <WxH>`
//! opens the window at that size instead of maximized, so one machine can
//! render several aspect ratios. `--settings-open` opens the Esc menu;
//! `--menu` (or `--menu-format FORMAT`) screenshots the main menu instead;
//! `--ui-size PERCENT` draws the UI at that size (0 = Auto); `--zoom-card
//! NAME` holds the camera close over that card on the viewer's board, for a
//! look at what sits on it (counter chips, badges); `--demo-damage` feeds the
//! client a batch of combat damage just before the screenshot, so it catches
//! the damage numerals in flight; `--stack` starts with two spells on the
//! stack; `--mana-gallery` lays every kind of mana symbol over the board;
//! `--hover-card NAME` hovers one of the viewer's battlefield cards (a
//! screenshot run ignores the real mouse); `--combat SCENE` stages a combat
//! or a targeting pick ([`CombatScene`]); `--tokens` adds piles of tokens
//! ([`add_token_piles`]); `--life-change` swings every seat's life total a
//! moment before the screenshot, catching the life feedback in flight.
//!
//!     cargo run --profile play -p crabomination_client -- \
//!         --layout-fixture 4 --window 1920x1080 --screenshot /tmp/pod.png

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use crabomination::card::CounterType;
use crabomination::game::{GameState, TurnStep};

/// Command-line options for the harness, parsed once in `main`.
#[derive(Resource, Clone, Debug, Default)]
pub struct HarnessArgs {
    /// Seats of the fixture board to boot into, if any.
    pub fixture_seats: Option<usize>,
    pub screenshot: Option<std::path::PathBuf>,
    pub screenshot_delay: f32,
    /// Forced window size (logical px); skips `maximize_window`.
    pub window: Option<(u32, u32)>,
    /// `--settings-open`: open the Esc menu once the view is up, so a
    /// screenshot can show it.
    pub settings_open: bool,
    /// `--viewer-out`: the viewer starts the fixture already knocked out
    /// (21 commander damage), for the "You're out" panel and the standings.
    pub viewer_out: bool,
    /// `--hold-seat N`: seat N is a connected player who never acts, so the
    /// game waits on them — a network pod that goes on without the viewer.
    pub hold_seat: Option<usize>,
    /// `--deck-picker`: stay on the menu with Commander selected and the deck
    /// picker open; the screenshot is of the menu, not a match.
    pub deck_picker: bool,
    /// `--import-report PATH`: import the decklist at PATH from the menu, as
    /// the decklist "From File" button would, so a screenshot shows the problems it finds.
    pub import_report: Option<std::path::PathBuf>,
    /// `--menu`: stay on the main menu; the screenshot is of it.
    /// `--menu-format FORMAT` does too, with that format selected
    /// (`commander` shows the pod options).
    pub menu: bool,
    pub menu_format: Option<crate::menu::MatchFormat>,
    /// `--ui-size PERCENT` (0 = Auto): the UI size for this run.
    pub ui_size: Option<u16>,
    /// `--zoom-card NAME`: the camera looks down on this card of the
    /// viewer's from close by.
    pub zoom_card: Option<String>,
    /// `--demo-damage`: a batch of damage events, client-side only, a moment
    /// before the screenshot.
    pub demo_damage: bool,
    /// `--stack`: the viewer has cast two spells and holds priority.
    pub stack: bool,
    /// `--mana-gallery`: a panel of sample costs over the board.
    pub mana_gallery: bool,
    /// `--hover-card NAME`: that card of the viewer's is hovered.
    pub hover_card: Option<String>,
    /// `--combat SCENE`: a combat or a targeting pick, staged client-side.
    pub combat: Option<CombatScene>,
    /// `--tokens`: piles of identical tokens on two seats.
    pub tokens: bool,
    /// `--life-change`: every seat's life total swings, client-side, a
    /// moment before the screenshot.
    pub life_change: bool,
}

impl HarnessArgs {
    pub fn parse(args: &[String]) -> Self {
        let value = |flag: &str| {
            args.windows(2).find_map(|w| (w[0] == flag).then(|| w[1].clone()))
        };
        HarnessArgs {
            fixture_seats: value("--layout-fixture")
                .and_then(|v| v.parse().ok())
                .map(|n: usize| n.clamp(2, 4)),
            screenshot: value("--screenshot").map(std::path::PathBuf::from),
            screenshot_delay: value("--screenshot-delay")
                .and_then(|v| v.parse().ok())
                .unwrap_or(8.0),
            window: value("--window").and_then(|v| {
                let (w, h) = v.split_once('x')?;
                Some((w.parse().ok()?, h.parse().ok()?))
            }),
            settings_open: args.iter().any(|a| a == "--settings-open"),
            viewer_out: args.iter().any(|a| a == "--viewer-out"),
            hold_seat: value("--hold-seat").and_then(|v| v.parse().ok()),
            deck_picker: args.iter().any(|a| a == "--deck-picker"),
            import_report: value("--import-report").map(std::path::PathBuf::from),
            menu: args.iter().any(|a| a == "--menu"),
            menu_format: value("--menu-format").and_then(|f| crate::menu::MatchFormat::from_cli(&f)),
            ui_size: value("--ui-size").and_then(|v| v.parse().ok()),
            zoom_card: value("--zoom-card"),
            demo_damage: args.iter().any(|a| a == "--demo-damage"),
            stack: args.iter().any(|a| a == "--stack"),
            mana_gallery: args.iter().any(|a| a == "--mana-gallery"),
            hover_card: value("--hover-card"),
            combat: value("--combat").and_then(|v| CombatScene::parse(&v)),
            tokens: args.iter().any(|a| a == "--tokens"),
            life_change: args.iter().any(|a| a == "--life-change"),
        }
    }

    /// The screenshot is of the menu, not a match.
    fn menu_shot(&self) -> bool {
        self.deck_picker || self.import_report.is_some() || self.menu || self.menu_format.is_some()
    }
}

/// Card names a fixture seat plays, by row. Looked up by name so a rename in
/// the catalog drops a card rather than breaking the harness.
const LANDS: &[&str] = &[
    "Forest", "Forest", "Forest", "Mountain", "Mountain", "Stomping Ground",
    "Wooded Foothills", "Island",
];
const CREATURES: &[&str] = &[
    "Llanowar Elves", "Grizzly Bears", "Tarmogoyf", "Serra Angel", "Shivan Dragon",
    "Birds of Paradise",
];
const OTHER_PERMANENTS: &[&str] = &["Sol Ring", "Oblivion Ring"];
/// Permanents that carry counters, and the counters they carry — every
/// counter surface (coins, their labels, the P/T and loyalty badges) on one
/// board. Two kinds on one card, and a stack past the coin cap.
const COUNTERED: &[(&str, &[(CounterType, u32)])] = &[
    ("Walking Ballista", &[(CounterType::PlusOnePlusOne, 11)]),
    ("Luminarch Aspirant", &[(CounterType::PlusOnePlusOne, 2), (CounterType::Stun, 1)]),
    ("Hangarback Walker", &[(CounterType::PlusOnePlusOne, 3)]),
    ("Jace, the Mind Sculptor", &[(CounterType::Loyalty, 5)]),
    ("History of Benalia", &[(CounterType::Lore, 2)]),
    ("Everflowing Chalice", &[(CounterType::Charge, 2)]),
];
const HAND: &[&str] = &[
    "Lightning Bolt", "Counterspell", "Wrath of God", "Grizzly Bears", "Forest",
    "Serra Angel", "Shivan Dragon",
];
const GRAVEYARD: &[&str] = &["Lightning Bolt", "Grizzly Bears", "Counterspell", "Forest"];

fn defs(names: &[&str]) -> Vec<crabomination::card::CardDefinition> {
    names.iter().filter_map(|n| crabomination::catalog::lookup_by_name(n)).collect()
}

/// A mid-game board for `seats` players (2 = a Modern 1v1, 3-4 = a Commander
/// pod), every seat with a full board, paused in seat 0's precombat main
/// phase with seat 0 holding priority — the match waits on the human, so the
/// table holds still for a screenshot.
pub fn fixture_state(seats: usize) -> GameState {
    let mut g = if seats <= 2 {
        crabomination::demo::build_demo_state_seeded(7)
    } else {
        let decks = crabomination::pod::pod_field(seats);
        let stock: Vec<crabomination::pod::SeatDeck<'_>> = decks
            .iter()
            .map(|d| crabomination::pod::SeatDeck { commanders: d.commanders, main: d.main })
            .collect();
        crabomination::demo::build_commander_pod_seeded(&stock, 7)
    };
    for seat in 0..g.players.len() {
        g.players[seat].name = if seat == 0 { "You".into() } else { format!("Bot {seat}") };
        for def in defs(LANDS) {
            g.add_card_to_battlefield(seat, def);
        }
        for (i, def) in defs(CREATURES).into_iter().enumerate() {
            let name = def.name;
            let id = g.add_card_to_battlefield(seat, def);
            g.clear_sickness(id);
            let Some(c) = g.battlefield_find_mut(id) else { continue };
            // A couple tapped, so the rotated-card spacing shows.
            c.tapped = i % 3 == 1;
            // A -1/-1 counter and some marked damage on the plain creatures.
            match name {
                "Grizzly Bears" => _ = c.counters.insert(CounterType::MinusOneMinusOne, 1),
                "Serra Angel" => c.damage = 2,
                _ => {}
            }
        }
        for def in defs(OTHER_PERMANENTS) {
            g.add_card_to_battlefield(seat, def);
        }
        for &(name, counters) in COUNTERED {
            let Some(def) = crabomination::catalog::lookup_by_name(name) else { continue };
            let id = g.add_card_to_battlefield(seat, def);
            g.clear_sickness(id);
            if let Some(c) = g.battlefield_find_mut(id) {
                for &(kind, n) in counters {
                    c.counters.insert(kind, n);
                }
            }
        }
        let hand = if seat == 0 { HAND.len() } else { 5 };
        for def in defs(&HAND[..hand]) {
            g.add_card_to_hand(seat, def);
        }
        for def in defs(GRAVEYARD) {
            g.add_card_to_graveyard(seat, def);
        }
    }
    for def in defs(&["Lightning Bolt", "Grizzly Bears"]) {
        g.add_card_to_exile(1, def);
    }
    g.players[1].poison_counters = 3;
    // An opposing monarch: the crown chip is a lone emoji on its row and
    // leads the viewer's "👑 <name>", so both of the symbol fallback's
    // lookups (Common and Latin runs) show up in a screenshot.
    if seats > 2 {
        g.monarch = Some(2);
    }
    g.turn_number = 6;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// `--stack`: seat 0 casts Lightning Bolt at seat 1's Serra Angel and, holding
/// priority, Giant Growth on its own Luminarch Aspirant — two items on the
/// stack, the pump on top, with the match waiting on the viewer.
pub fn put_spells_on_stack(g: &mut GameState) {
    use crabomination::game::{GameAction, Target};
    let find = |g: &GameState, seat: usize, name: &str| {
        g.battlefield.iter().find(|c| c.controller == seat && c.definition.name == name).map(|c| c.id)
    };
    let casts = [("Lightning Bolt", find(g, 1, "Serra Angel")), ("Giant Growth", find(g, 0, "Luminarch Aspirant"))];
    for (spell, target) in casts {
        let Some(def) = crabomination::catalog::lookup_by_name(spell) else { continue };
        let card_id = g.add_card_to_hand(0, def);
        let cast = GameAction::CastSpell {
            card_id,
            target: target.map(Target::Permanent),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        if let Err(e) = g.perform_action(cast) {
            eprintln!("--stack: casting {spell}: {e:?}");
        }
    }
}

/// `--tokens`: seven Goblins and three tapped ones, two Soldiers and four
/// Treasures for seat 0, five Spirits for seat 1 — piles of each size a
/// token-making deck leaves, in both rows.
pub fn add_token_piles(g: &mut GameState) {
    use crabomination::card::{CardType, Keyword, TokenDefinition};
    let creature = |name: &str, keywords: Vec<Keyword>| TokenDefinition {
        name: name.into(),
        power: 1,
        toughness: 1,
        keywords,
        card_types: vec![CardType::Creature],
        ..Default::default()
    };
    let treasure = TokenDefinition { name: "Treasure".into(), card_types: vec![CardType::Artifact], ..Default::default() };
    let piles = [
        (0, creature("Goblin", vec![]), 10),
        (0, creature("Soldier", vec![]), 2),
        (0, treasure, 4),
        (1, creature("Spirit", vec![Keyword::Flying]), 5),
    ];
    for (seat, token, n) in piles {
        for i in 0..n {
            let id = g.add_token_to_battlefield(seat, &token);
            g.clear_sickness(id);
            // Three of the Goblins have attacked: a tapped pile beside the
            // untapped one.
            if token.name == "Goblin" && i >= 7 && let Some(c) = g.battlefield_find_mut(id) {
                c.tapped = true;
            }
        }
    }
}

/// `--viewer-out`: knock seat 0 out of the fixture with 21 damage from
/// seat 1's commander, and hand the turn to seat 1.
pub fn knock_out_viewer(g: &mut GameState) {
    let Some(&commander) = g.players.get(1).and_then(|p| p.commanders.first()) else { return };
    g.commander_damage.insert((0, commander), 21);
    g.check_state_based_actions();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
}

/// `--deck-picker`: select Commander and open the deck picker, once.
/// `--menu-format FORMAT`: select that format, once.
pub fn open_deck_picker_for_screenshot(
    args: Res<HarnessArgs>,
    mut fields: ResMut<crate::menu::MenuFields>,
    mut picker: ResMut<crate::deck_picker::DeckPicker>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    *done = true;
    if args.deck_picker {
        fields.select_format(crate::menu::MatchFormat::Commander);
        picker.open = true;
    } else if let Some(format) = args.menu_format {
        fields.select_format(format);
    }
}

/// `--import-report PATH`: import that list from the menu, once.
pub fn import_for_screenshot(
    args: Res<HarnessArgs>,
    fields: Res<crate::menu::MenuFields>,
    mut status: ResMut<crate::menu::MenuStatus>,
    mut report: ResMut<crate::deck_import::ImportReport>,
    mut done: Local<bool>,
) {
    let Some(path) = args.import_report.as_ref().filter(|_| !*done) else { return };
    *done = true;
    let text = std::fs::read_to_string(path).unwrap_or_default();
    if let Err(problems) = crate::deck_import::import_deck(&text, fields.format) {
        status.0 = problems.summary.clone();
        report.0 = Some(problems);
    }
}

/// `--settings-open`: open the Esc menu the first frame a view is up.
pub fn open_settings_for_screenshot(
    args: Res<HarnessArgs>,
    view: Res<crate::net_plugin::CurrentView>,
    mut settings: ResMut<crate::systems::quality::SettingsOpen>,
    mut done: Local<bool>,
) {
    if args.settings_open && !*done && view.0.is_some() {
        settings.0 = true;
        *done = true;
    }
}

/// `--zoom-card NAME`: put the camera close over that card, at the home
/// pose's angle. Runs after `camera_zoom`, which would ease it home.
pub fn zoom_on_card_for_screenshot(
    args: Res<HarnessArgs>,
    view: Res<crate::net_plugin::CurrentView>,
    home: Res<crate::systems::camera_zoom::CameraHome>,
    cards: Query<(&crate::card::GameCardId, &GlobalTransform), With<crate::card::BattlefieldCard>>,
    mut camera: Query<&mut Transform, With<crate::MainCamera>>,
) {
    let (Some(name), Some(cv)) = (args.zoom_card.as_deref(), view.0.as_ref()) else { return };
    let Some(id) = cv.battlefield.iter().find(|p| p.controller == cv.your_seat && p.name == name).map(|p| p.id)
    else {
        return;
    };
    let Some((_, card)) = cards.iter().find(|(g, _)| g.0 == id) else { return };
    let Ok(mut transform) = camera.single_mut() else { return };
    let focus = card.translation();
    *transform = Transform::from_translation(focus + (home.pose.translation - home.target) * 0.16)
        .looking_at(focus, Vec3::Y);
}

/// Hits `--demo-damage` deals: (seat, card, damage). Luminarch Aspirant is
/// hit twice in one batch, as by two blockers.
const DEMO_HITS: &[(usize, &str, u32)] = &[
    (1, "Serra Angel", 3),
    (1, "Walking Ballista", 5),
    (0, "Luminarch Aspirant", 2),
    (0, "Luminarch Aspirant", 1),
    (0, "Hangarback Walker", 4),
];

/// `--demo-damage`: add [`DEMO_HITS`] to the event batch once, a third of a
/// second before the screenshot — the view itself is untouched, so only the
/// transient effects (numerals, sparks) show. Runs after `poll_net`, which
/// clears the batch each frame.
pub fn inject_damage_for_screenshot(
    args: Res<HarnessArgs>,
    view: Res<crate::net_plugin::CurrentView>,
    time: Res<Time>,
    mut events: ResMut<crate::net_plugin::LatestServerEvents>,
    mut since_view: Local<f32>,
    mut done: Local<bool>,
) {
    let Some(cv) = view.0.as_ref().filter(|_| args.demo_damage && !*done) else { return };
    *since_view += time.delta_secs();
    if *since_view < args.screenshot_delay - 0.3 {
        return;
    }
    *done = true;
    for &(seat, name, amount) in DEMO_HITS {
        if let Some(p) = cv.battlefield.iter().find(|p| p.controller == seat && p.name == name) {
            events.0.push(crabomination::net::GameEventWire::DamageDealt {
                amount,
                to_player: None,
                to_card: Some(p.id),
            });
        }
    }
}

/// What `--mana-gallery` shows: each kind of symbol the engine writes, in
/// the shapes the client prints them.
const MANA_GALLERY: &[&str] = &[
    "{3}{W}{W}   {X}{R}{R}   {1}{U}{B}{G}   {10}",
    "{2}{T}: Draw a card.   {Q}: Untap target land.",
    "Hybrid {W/U}{W/U}   {2/G}   {C/W}   Phyrexian {B/P}   {R/G/P}",
    "{C}{C}   Snow {S}   Energy {E}{E}",
    "{4}{G} (+2 tax)",
    "Ward {2}   Kicker {1}{R}   {1}{U} to go · tap mana sources",
];

/// `--mana-gallery`: spawn [`MANA_GALLERY`] once, at 13 and 18 px, over the
/// top of the board.
pub fn spawn_mana_gallery(
    mut commands: Commands,
    args: Res<HarnessArgs>,
    view: Res<crate::net_plugin::CurrentView>,
    ui_fonts: Res<crate::theme::UiFonts>,
    mut done: Local<bool>,
) {
    if !args.mana_gallery || *done || view.0.is_none() {
        return;
    }
    *done = true;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(30.0),
                top: Val::Px(120.0),
                max_width: Val::Px(620.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(crate::theme::PANEL_BG),
            GlobalZIndex(crate::theme::layer::MODAL),
            crate::systems::game_ui::InGameRoot,
        ))
        .with_children(|panel| {
            for size in [13.0, 18.0] {
                for line in MANA_GALLERY {
                    panel.spawn(crate::mana_text::mana_text(*line, size, crate::theme::TEXT_PRIMARY));
                }
            }
        });
    // A real decision modal with costs in its title and options, to see
    // them lay out (and wrap) where a game shows them.
    let ballot = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(30.0),
                top: Val::Px(560.0),
                // As `decision_ui::spawn_modal_panel`: a floor, not a cap.
                min_width: Val::Px(380.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                padding: UiRect::all(Val::Px(20.0)),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(crate::theme::PANEL_BG),
            GlobalZIndex(crate::theme::layer::MODAL),
            crate::systems::game_ui::InGameRoot,
        ))
        .id();
    crate::systems::decision_ui::fill_option_ballot(
        &mut commands,
        ballot,
        &ui_fonts,
        "Kitesail Freebooter — pay {2} for ward, or the spell is countered",
        &[
            "Pay {2}".to_string(),
            "Pay {1}{U} and sacrifice a creature, then draw a card and scry {E}{E}".to_string(),
            "Decline".to_string(),
        ],
    );
}

/// `--hover-card NAME`: hover that card as the pointer would
/// (`card::observers::on_card_over`), once its entity is up and settled.
pub fn hover_card_for_screenshot(
    mut commands: Commands,
    args: Res<HarnessArgs>,
    view: Res<crate::net_plugin::CurrentView>,
    mut cards: Query<
        (Entity, &crate::card::GameCardId, &Transform, &mut crate::card::CardHoverLift),
        (With<crate::card::BattlefieldCard>, Without<crate::card::Animating>),
    >,
    mut done: Local<bool>,
) {
    let (Some(name), Some(cv)) = (args.hover_card.as_deref(), view.0.as_ref()) else { return };
    if *done {
        return;
    }
    let Some(id) = cv.battlefield.iter().find(|p| p.controller == cv.your_seat && p.name == name).map(|p| p.id)
    else {
        return;
    };
    let Some((entity, _, transform, mut lift)) = cards.iter_mut().find(|(_, g, ..)| g.0 == id) else { return };
    lift.base_translation = transform.translation - Vec3::Y * lift.current_lift;
    lift.target_lift = crate::card::BF_HOVER_LIFT;
    commands.entity(entity).insert(crate::card::CardHovered);
    *done = true;
}

/// How each seat's life swings under `--life-change`: the viewer loses 3,
/// seat 1 loses 5, seat 2 gains 4 and seat 3 loses 12.
const LIFE_SWINGS: [i32; 4] = [-3, -5, 4, -12];

/// `--life-change`: swing each seat's life by [`LIFE_SWINGS`] once, in the
/// view only, most of a second before the screenshot — the count and the
/// floating numerals are caught part-way. Runs after `poll_net`.
pub fn swing_life_for_screenshot(
    args: Res<HarnessArgs>,
    time: Res<Time>,
    mut view: ResMut<crate::net_plugin::CurrentView>,
    mut since_view: Local<f32>,
    mut done: Local<bool>,
) {
    if !args.life_change || *done || view.0.is_none() {
        return;
    }
    *since_view += time.delta_secs();
    if *since_view < args.screenshot_delay - 0.35 {
        return;
    }
    *done = true;
    if let Some(cv) = view.0.as_mut() {
        for (p, swing) in cv.players.iter_mut().zip(LIFE_SWINGS) {
            p.life += swing;
        }
    }
}

/// What `--combat` stages. The match itself stays paused in the viewer's
/// main phase: each view that arrives is patched, the viewer's plans are
/// set, and auto-pass is held, so nothing moves on before the screenshot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CombatScene {
    /// `blocks`: seat 1 attacks the viewer, who is picking blocks — one
    /// block planned, another blocker picked up.
    Blocks,
    /// `declared`: the viewer has attacked (in a pod, three seats at once)
    /// and the defenders have blocked two of the attackers.
    Declared,
    /// `plan`: the viewer is picking attackers.
    Plan,
    /// `target`: the viewer is casting Lightning Bolt and pointing it at
    /// seat 1's Serra Angel.
    Target,
}

impl CombatScene {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "blocks" => Some(Self::Blocks),
            "declared" => Some(Self::Declared),
            "plan" => Some(Self::Plan),
            "target" => Some(Self::Target),
            _ => None,
        }
    }
}

/// The creatures that attack in a `--combat` scene; Serra Angel has
/// vigilance, so it attacks untapped.
const ATTACKERS: [&str; 3] = ["Tarmogoyf", "Shivan Dragon", "Serra Angel"];

/// Who the `i`th attacker attacks in a `--combat declared` scene: seat 1 in
/// a duel, a different opponent each in a pod.
fn defender_for(i: usize, seats: usize) -> usize {
    if seats <= 2 { 1 } else { 1 + i % (seats - 1) }
}

fn permanent_id(cv: &crabomination::net::ClientView, seat: usize, name: &str) -> Option<crabomination::card::CardId> {
    cv.battlefield.iter().find(|p| p.controller == seat && p.name == name).map(|p| p.id)
}

/// Patch a view into `scene`'s step and attacks.
fn stage_view(cv: &mut crabomination::net::ClientView, scene: CombatScene) {
    use crabomination::game::AttackTarget;
    let (viewer, seats) = (cv.your_seat, cv.players.len());
    // Priority sits with a bot wherever the scene allows, which holds the
    // client's auto-pass on its own.
    let (step, active, priority) = match scene {
        CombatScene::Blocks => (TurnStep::DeclareBlockers, 1, 1),
        CombatScene::Declared => (TurnStep::DeclareBlockers, viewer, 1),
        CombatScene::Plan => (TurnStep::DeclareAttackers, viewer, viewer),
        CombatScene::Target => (TurnStep::PreCombatMain, viewer, viewer),
    };
    cv.step = step;
    cv.active_player = active;
    cv.priority = priority;
    let (attacker_seat, defender): (usize, fn(usize, usize) -> usize) = match scene {
        CombatScene::Blocks => (1, |_, _| 0),
        CombatScene::Declared => (viewer, defender_for),
        CombatScene::Plan | CombatScene::Target => return,
    };
    for (i, name) in ATTACKERS.iter().enumerate() {
        let seat = defender(i, seats);
        let Some(p) = cv.battlefield.iter_mut().find(|p| p.controller == attacker_seat && p.name == *name) else {
            continue;
        };
        p.attacking = true;
        p.tapped = *name != "Serra Angel";
        p.attack_target = Some(AttackTarget::Player(seat));
        p.defending_player = Some(seat);
    }
    if scene == CombatScene::Declared {
        // Each of the first two attackers' defenders blocks it.
        for (i, blocker) in [(0, "Tarmogoyf"), (1, "Serra Angel")] {
            let Some(attacker) = permanent_id(cv, viewer, ATTACKERS[i]) else { continue };
            let seat = defender_for(i, seats);
            if let Some(b) = cv.battlefield.iter_mut().find(|p| p.controller == seat && p.name == blocker) {
                b.blocking_attackers = vec![attacker];
            }
        }
    }
}

/// `--combat SCENE`: stage [`CombatScene`] — patch each view that arrives,
/// set the viewer's attack, block or targeting plan, and hold auto-pass.
/// Runs after `poll_net`.
#[allow(clippy::too_many_arguments)]
pub fn stage_combat_for_screenshot(
    mut commands: Commands,
    args: Res<HarnessArgs>,
    mut view: ResMut<crate::net_plugin::CurrentView>,
    (mut blocking, mut attacking): (ResMut<crate::game::BlockingState>, ResMut<crate::game::AttackingState>),
    (mut targeting, mut legal): (ResMut<crate::game::TargetingState>, ResMut<crate::game::LegalTargets>),
    mut ff: ResMut<crate::systems::game_ui::FastForward>,
    cards: Query<(Entity, &crate::card::GameCardId), With<crate::card::BattlefieldCard>>,
    mut hovered: Local<bool>,
) {
    use crabomination::game::AttackTarget;
    let Some(scene) = args.combat else { return };
    if view.is_changed()
        && let Some(cv) = view.bypass_change_detection().0.as_mut()
    {
        stage_view(cv, scene);
    }
    let Some(cv) = view.0.as_ref() else { return };
    if !ff.manual_priority {
        ff.manual_priority = true;
    }
    let (viewer, seats) = (cv.your_seat, cv.players.len());
    let mine = |name: &str| permanent_id(cv, viewer, name);
    match scene {
        CombatScene::Blocks => {
            let theirs = |name: &str| permanent_id(cv, 1, name);
            let assignments: Vec<_> = mine("Tarmogoyf").zip(theirs("Tarmogoyf")).into_iter().collect();
            let selected = mine("Serra Angel");
            if blocking.assignments != assignments || blocking.selected_blocker != selected {
                blocking.assignments = assignments;
                blocking.selected_blocker = selected;
            }
        }
        CombatScene::Plan => {
            let plan: Vec<_> = ATTACKERS
                .iter()
                .enumerate()
                .filter_map(|(i, name)| Some((mine(name)?, AttackTarget::Player(defender_for(i + 1, seats)))))
                .collect();
            if attacking.plan != plan {
                attacking.last_added = plan.last().map(|(a, _)| *a);
                attacking.plan = plan;
            }
        }
        CombatScene::Target => {
            let bolt = cv.players.get(viewer).and_then(|p| {
                p.hand.iter().find_map(|h| match h {
                    crabomination::net::HandCardView::Known(k) if k.name == "Lightning Bolt" => Some(k.id),
                    _ => None,
                })
            });
            if !targeting.active || targeting.pending_card_id != bolt {
                targeting.active = true;
                targeting.pending_card_id = bolt;
            }
            if legal.permanents.is_empty() {
                legal.permanents = cv
                    .battlefield
                    .iter()
                    .filter(|p| p.controller != viewer && p.power > 0)
                    .map(|p| p.id)
                    .collect();
                legal.enumerated = true;
            }
            // Point at the target as the pointer would, once its card is up.
            if !*hovered
                && let Some(angel) = permanent_id(cv, 1, "Serra Angel")
                && let Some((e, _)) = cards.iter().find(|(_, g)| g.0 == angel)
            {
                commands.entity(e).insert(crate::card::CardHovered);
                *hovered = true;
            }
        }
        CombatScene::Declared => {}
    }
}

/// A screenshot run ignores the mouse: the desktop's cursor, wherever it
/// happens to sit over the window, hovered a card and popped its preview
/// into the shot.
pub fn ignore_mouse_for_screenshot(
    args: Res<HarnessArgs>,
    mut pointer: ResMut<bevy::picking::input::PointerInputSettings>,
) {
    if args.screenshot.is_some() {
        pointer.is_mouse_enabled = false;
    }
}

/// Seconds since the first view arrived; `None` until then.
#[derive(Resource, Default)]
pub struct ScreenshotClock {
    since_view: Option<f32>,
    taken_at: Option<f32>,
}

/// Save the primary window to `--screenshot` once the view has been up for
/// the delay, then exit a moment later (the capture lands a frame or two
/// after the request).
pub fn capture_screenshot(
    mut commands: Commands,
    args: Res<HarnessArgs>,
    view: Res<crate::net_plugin::CurrentView>,
    time: Res<Time>,
    mut clock: ResMut<ScreenshotClock>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(path) = &args.screenshot else { return };
    // A match screenshot waits for the first view; a menu one does not.
    if view.0.is_none() && !args.menu_shot() {
        return;
    }
    let t = clock.since_view.get_or_insert(0.0);
    *t += time.delta_secs();
    let now = *t;
    match clock.taken_at {
        None if now >= args.screenshot_delay => {
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path.clone()));
            clock.taken_at = Some(now);
        }
        Some(at) if now - at > 2.0 => {
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harness_flags_parse() {
        let args: Vec<String> = ["--layout-fixture", "4", "--window", "2560x1080", "--screenshot", "/tmp/a.png"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let h = HarnessArgs::parse(&args);
        assert_eq!(h.fixture_seats, Some(4));
        assert_eq!(h.window, Some((2560, 1080)));
        assert_eq!(h.screenshot.as_deref(), Some(std::path::Path::new("/tmp/a.png")));
        assert_eq!(h.screenshot_delay, 8.0);
        assert!(HarnessArgs::parse(&[]).fixture_seats.is_none());
        let out: Vec<String> = ["--viewer-out", "--hold-seat", "1"].iter().map(|s| s.to_string()).collect();
        let h = HarnessArgs::parse(&out);
        assert!(h.viewer_out);
        assert_eq!(h.hold_seat, Some(1));
    }

    #[test]
    fn fixture_boards_are_full_and_paused_on_the_viewer() {
        for seats in [2, 4] {
            let g = fixture_state(seats);
            assert_eq!(g.players.len(), seats);
            for p in 0..seats {
                let on_board = g.battlefield.iter().filter(|c| c.controller == p).count();
                assert!(on_board >= 14, "seat {p} of {seats}: {on_board} permanents");
                assert!(!g.players[p].hand.is_empty());
            }
            assert_eq!((g.step, g.active_player_idx), (TurnStep::PreCombatMain, 0));
        }
    }
}
