//! Puzzle mode: the blunder benchmark's positions (`server::blunder`) as
//! playable challenges. Each is a board where the bots once went wrong —
//! hold the counterspell, save the removal for the bigger threat, find the
//! lethal attack — with the checks that score the right play.
//!
//! Main menu → **Puzzles** lists them; picking one starts a local match
//! from its position with you in seat 0. When its window closes (the turn
//! horizon passes, or the game ends) the result banner says whether you
//! solved it, and what the checks wanted where you didn't.
//!
//! The checks are judged here from the match's own events — what was cast,
//! at what, which creatures attacked and blocked, which abilities were
//! activated — and the final board, so a rejected submission never counts.

use bevy::prelude::*;
use crabomination::card::CardId;
use crabomination::net::GameEventWire;
use crabomination::server::blunder::{self, Check};

use crate::net_plugin::{CurrentView, LatestServerEvents};
use crate::theme::{self, HoverTint, UiFonts};
use crate::ui_button::{Button, Interaction};

/// What the viewer did this puzzle, read off the events.
#[derive(Default, Debug, Clone)]
pub struct Facts {
    casts: Vec<CardId>,
    /// `(card, permanent it targeted)`, paired within one batch.
    cast_targets: Vec<(CardId, CardId)>,
    attacks: Vec<CardId>,
    blocks: Vec<CardId>,
    activations: Vec<CardId>,
}

impl Facts {
    /// Fold one batch of events: only the viewer's casts (and the targets
    /// they named), and attacks / blocks / activations of the viewer's
    /// permanents.
    pub fn fold(&mut self, events: &[GameEventWire], viewer: usize, mine: impl Fn(CardId) -> bool) {
        let mut cast_now = Vec::new();
        for ev in events {
            match ev {
                GameEventWire::SpellCast { player, card_id, .. } if *player == viewer => {
                    self.casts.push(*card_id);
                    cast_now.push(*card_id);
                }
                GameEventWire::BecameTarget { target, caster } if *caster == viewer => {
                    for c in &cast_now {
                        self.cast_targets.push((*c, *target));
                    }
                }
                GameEventWire::AttackerDeclared(id) if mine(*id) => self.attacks.push(*id),
                GameEventWire::BlockerDeclared { blocker, .. } if mine(*blocker) => self.blocks.push(*blocker),
                GameEventWire::AbilityActivated { source, .. } if mine(*source) => self.activations.push(*source),
                GameEventWire::LoyaltyAbilityActivated { planeswalker, .. } if mine(*planeswalker) => {
                    self.activations.push(*planeswalker)
                }
                _ => {}
            }
        }
    }
}

/// Whether `check` holds for the viewer (seat 0) given what they did and
/// the final view — the client's reading of `blunder::Check`.
pub fn holds(check: Check, facts: &Facts, cv: &crabomination::net::ClientView) -> bool {
    let on_board = |id: CardId| cv.battlefield.iter().any(|p| p.id == id);
    let you = cv.your_seat;
    let lost = |seat: usize| {
        cv.game_over == Some(Some(1 - seat)) || cv.players.iter().find(|p| p.seat == seat).is_none_or(|p| p.eliminated)
    };
    match check {
        Check::Attacks(id) => facts.attacks.contains(&id),
        Check::NeverAttacks(id) => !facts.attacks.contains(&id),
        Check::Blocks(id) => facts.blocks.contains(&id),
        Check::NeverBlocks(id) => !facts.blocks.contains(&id),
        Check::Dies(id) => !on_board(id),
        Check::Survives(id) => on_board(id),
        Check::Activates(id) => facts.activations.contains(&id),
        Check::Casts(id) => facts.casts.contains(&id),
        Check::CastOn(id, at) => facts.cast_targets.contains(&(id, at)),
        Check::NeverCasts(id) => !facts.casts.contains(&id),
        Check::Wins => cv.game_over == Some(Some(you)) || lost(1 - you),
        Check::Alive => !lost(you),
    }
}

/// The puzzle being played.
#[derive(Resource, Default)]
pub struct PuzzleRun {
    pub active: Option<ActivePuzzle>,
}

pub struct ActivePuzzle {
    pub prompt: &'static str,
    checks: Vec<(Check, String)>,
    start_turn: u32,
    horizon: u32,
    facts: Facts,
    /// `None` while playing; the checks missed once judged (empty = solved).
    pub result: Option<Vec<String>>,
}

impl ActivePuzzle {
    /// A run of `item`, with its checks described off its start position.
    pub fn new(item: &blunder::Blunder) -> (Self, crabomination::game::GameState) {
        let pos = (item.build)();
        let checks = pos.checks.iter().map(|c| (*c, c.describe(&pos.state))).collect();
        let run = ActivePuzzle {
            prompt: item.prompt,
            checks,
            start_turn: pos.state.turn_number,
            horizon: item.horizon,
            facts: Facts::default(),
            result: None,
        };
        (run, pos.state)
    }
}

/// Fold events into the run, and judge it once its window closes.
pub fn track_puzzle(events: Res<LatestServerEvents>, view: Res<CurrentView>, mut run: ResMut<PuzzleRun>) {
    let Some(p) = run.active.as_mut() else { return };
    let Some(cv) = &view.0 else { return };
    if p.result.is_some() {
        return;
    }
    if !events.0.is_empty() {
        let mine = |id: CardId| cv.battlefield.iter().any(|c| c.id == id && c.controller == cv.your_seat);
        p.facts.fold(&events.0, cv.your_seat, mine);
    }
    if cv.game_over.is_some() || cv.turn > p.start_turn + p.horizon {
        p.result = Some(p.checks.iter().filter(|(c, _)| !holds(*c, &p.facts, cv)).map(|(_, d)| d.clone()).collect());
    }
}

/// A benchmark prompt as a puzzle's question and its answer: the items are
/// written "situation — answer" for the benchmark's report, and a puzzle
/// that printed the answer would be no puzzle.
pub fn split_prompt(prompt: &str) -> (&str, Option<&str>) {
    match prompt.split_once(" — ") {
        Some((q, a)) => (q, Some(a)),
        None => (prompt, None),
    }
}

#[derive(Component)]
pub struct PuzzleBanner;

#[derive(Component)]
pub struct PuzzleBannerText;

/// The banner: the puzzle's question while it runs, the verdict after.
pub fn sync_puzzle_banner(
    mut commands: Commands,
    run: Res<PuzzleRun>,
    ui_fonts: Res<UiFonts>,
    banner: Query<Entity, With<PuzzleBanner>>,
    mut text: Query<(&mut Text, &mut TextColor), With<PuzzleBannerText>>,
) {
    let Some(p) = run.active.as_ref() else {
        for e in &banner {
            commands.entity(e).despawn();
        }
        return;
    };
    let (question, answer) = split_prompt(p.prompt);
    let (line, colour) = match &p.result {
        None => (format!("Puzzle: {question}. Find the right play."), theme::ACCENT_GOLD),
        Some(missed) if missed.is_empty() => match answer {
            Some(a) => (format!("Solved: {a}."), theme::TEXT_GOOD),
            None => ("Solved.".to_string(), theme::TEXT_GOOD),
        },
        Some(missed) => (format!("Not quite: the right play was to {}.", missed.join(", ")), theme::TEXT_DANGER),
    };
    if let Ok((mut t, mut c)) = text.single_mut() {
        if t.0 != line {
            t.0 = line;
            c.0 = colour;
        }
        return;
    }
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(200.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            PuzzleBanner,
            crate::systems::game_ui::InGameRoot,
            Pickable::IGNORE,
            GlobalZIndex(theme::layer::BANNER),
        ))
        .with_children(|row| {
            row.spawn((
                Node {
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                    max_width: Val::Px(760.0),
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(theme::HUD_BG),
                Pickable::IGNORE,
            ))
            .with_children(|b| {
                b.spawn((Text::new(line), ui_fonts.tf(15.0), TextColor(colour), PuzzleBannerText, Pickable::IGNORE));
            });
        });
}

// ── The menu's list ──────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct PuzzlesOpen(pub bool);

#[derive(Component)]
pub struct OpenPuzzlesButton;

#[derive(Component)]
pub struct PuzzlesPanelRoot;

/// A puzzle's row: its index in `blunder::items()`.
#[derive(Component)]
pub struct PuzzleRow(pub usize);

#[derive(Component)]
pub struct PuzzlesCloseButton;

/// The puzzle a row click picked, for the menu to start.
#[derive(Resource, Default)]
pub struct PickedPuzzle(pub Option<usize>);

pub fn handle_puzzles_panel(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut open: ResMut<PuzzlesOpen>,
    mut picked: ResMut<PickedPuzzle>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<OpenPuzzlesButton>)>,
    closes: Query<&Interaction, (Changed<Interaction>, With<PuzzlesCloseButton>)>,
    rows: Query<(&Interaction, &PuzzleRow), Changed<Interaction>>,
) {
    if buttons.iter().any(|i| *i == Interaction::Pressed) {
        open.0 = !open.0;
    }
    if open.0 && (keyboard.just_pressed(KeyCode::Escape) || closes.iter().any(|i| *i == Interaction::Pressed)) {
        open.0 = false;
    }
    for (i, row) in &rows {
        if *i == Interaction::Pressed {
            picked.0 = Some(row.0);
            open.0 = false;
        }
    }
}

pub fn sync_puzzles_panel(
    mut commands: Commands,
    open: Res<PuzzlesOpen>,
    ui_fonts: Res<UiFonts>,
    existing: Query<Entity, With<PuzzlesPanelRoot>>,
) {
    if !open.is_changed() {
        return;
    }
    for e in &existing {
        commands.entity(e).despawn();
    }
    if !open.0 {
        return;
    }
    let tf = |s: f32| ui_fonts.tf(s);
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(theme::OVERLAY_BG_HEAVY),
            PuzzlesPanelRoot,
            GlobalZIndex(theme::layer::TOP_PROMPT),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(8.0),
                    width: Val::Px(720.0),
                    max_height: Val::Percent(90.0),
                    border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                    ..default()
                },
                BackgroundColor(theme::PANEL_BG),
            ))
            .with_children(|outer| {
                outer.spawn((Text::new("Puzzles"), tf(20.0), TextColor(theme::ACCENT_GOLD), Node { align_self: AlignSelf::Center, ..default() }));
                outer.spawn((
                    Text::new("Boards where the bots once went wrong. Find the right play; the banner judges it when the turn window closes."),
                    tf(12.0),
                    TextColor(theme::TEXT_SECONDARY),
                ));
                outer
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            overflow: Overflow::scroll_y(),
                            min_height: Val::Px(0.0),
                            flex_shrink: 1.0,
                            ..default()
                        },
                        bevy::ui_widgets::ScrollArea,
                    ))
                    .with_children(|list| {
                        for (i, item) in blunder::items().iter().enumerate() {
                            list.spawn((
                                Button,
                                Node { padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)), border_radius: BorderRadius::all(theme::RADIUS_BUTTON), ..default() },
                                BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                                HoverTint::new(theme::BUTTON_NEUTRAL_BG),
                                PuzzleRow(i),
                            ))
                            .with_children(|b| {
                                let question = split_prompt(item.prompt).0;
                                b.spawn((Text::new(format!("{}. {question}", i + 1)), tf(13.0), TextColor(theme::TEXT_PRIMARY), Pickable::IGNORE));
                            });
                        }
                    });
                outer
                    .spawn((
                        Button,
                        Node { padding: UiRect::axes(Val::Px(18.0), Val::Px(8.0)), align_self: AlignSelf::Center, border_radius: BorderRadius::all(theme::RADIUS_BUTTON), ..default() },
                        BackgroundColor(theme::BUTTON_PRIMARY_BG),
                        HoverTint::new(theme::BUTTON_PRIMARY_BG),
                        PuzzlesCloseButton,
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new("Close (Esc)"), tf(14.0), TextColor(theme::TEXT_PRIMARY), Pickable::IGNORE));
                    });
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every benchmark item builds, names its checks, and starts on a turn
    /// the run can count from.
    #[test]
    fn every_puzzle_builds() {
        let items = blunder::items();
        assert!(!items.is_empty());
        for item in &items {
            let (run, state) = ActivePuzzle::new(item);
            assert_eq!(state.players.len(), 2, "{}", item.id);
            assert!(!run.checks.is_empty(), "{} has no checks", item.id);
            assert!(run.checks.iter().all(|(_, d)| !d.contains('?')), "{}: {:?}", item.id, run.checks);
        }
    }

    #[test]
    fn a_question_hides_its_answer() {
        assert_eq!(split_prompt("4/4 vs untapped Typhoid Rats — don't attack"), ("4/4 vs untapped Typhoid Rats", Some("don't attack")));
        assert_eq!(split_prompt("two lands, a land and a three-drop in hand"), ("two lands, a land and a three-drop in hand", None));
    }

    /// The events decide a cast and its target; a rejected submission never
    /// appears among them.
    #[test]
    fn facts_come_from_the_events() {
        let (bolt, bear) = (CardId(1), CardId(2));
        let mut f = Facts::default();
        f.fold(
            &[
                GameEventWire::SpellCast { player: 0, card_id: bolt, face: Default::default() },
                GameEventWire::BecameTarget { target: bear, caster: 0 },
                GameEventWire::AttackerDeclared(CardId(5)),
            ],
            0,
            |id| id == CardId(5),
        );
        assert_eq!(f.casts, vec![bolt]);
        assert_eq!(f.cast_targets, vec![(bolt, bear)]);
        assert_eq!(f.attacks, vec![CardId(5)]);
    }
}
