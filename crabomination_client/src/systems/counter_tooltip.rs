//! Alt-key tooltip showing counter detail and modified P/T for the
//! battlefield card under the cursor.
//!
//! Hold either Alt (left/right) while hovering a card to surface a small
//! HUD panel with:
//!
//! - Current power/toughness (if creature) and `(printed X/Y)`
//!   when the values differ from the printed P/T.
//! - Loyalty count (for planeswalkers).
//! - One row per counter type and quantity (`+1/+1 ×3`, `Stun ×2`, …).
//!
//! The 3-D counter coins handle the at-a-glance "this card has stuff on
//! it" indicator; this tooltip is the click-through for the details
//! a player needs when the coin column gets dense.
//!
//! Anchored to the bottom-right corner of the viewport rather than
//! floating next to the 3-D card, because the peek popup
//! (`systems::ui::peek_popup`) also lights up on Alt-hold and centers a
//! large card-art image — a card-adjacent tooltip would overlap it.

use bevy::prelude::*;
use crabomination::card::{CardId, CardType, CounterType};

use crate::card::{BattlefieldCard, CardHovered, GameCardId};
use crate::net_plugin::CurrentView;
use crate::theme::UiFonts;

/// Root marker for the floating tooltip panel.
#[derive(Component)]
pub struct AltTooltip;

/// Marker on the tooltip's text node so the update system can rewrite it
/// without doing a child walk.
#[derive(Component)]
pub struct AltTooltipText;

#[allow(clippy::too_many_arguments)]
pub fn update_alt_tooltip(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    hovered: Query<&GameCardId, (With<BattlefieldCard>, With<CardHovered>)>,
    mut tooltip_q: Query<Entity, With<AltTooltip>>,
    mut text_q: Query<&mut Text, With<AltTooltipText>>,
) {
    let alt_held = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);

    // No alt or no hovered card → tear down any tooltip.
    let hovered_card_id: Option<CardId> = if alt_held {
        hovered.iter().next().map(|gid| gid.0)
    } else {
        None
    };

    let Some(card_id) = hovered_card_id else {
        for e in tooltip_q.iter() {
            commands.entity(e).despawn();
        }
        return;
    };

    let Some(cv) = &view.0 else { return };
    let Some(p) = cv.battlefield.iter().find(|p| p.id == card_id) else {
        // Card left the battlefield — drop the tooltip.
        for e in tooltip_q.iter() {
            commands.entity(e).despawn();
        }
        return;
    };

    // Build the body without the card name (the peek popup already
    // shows the card art with its name). If there's nothing
    // interesting (no P/T mod, no loyalty, no counters, not tapped),
    // skip the tooltip entirely so we don't render an empty panel.
    let Some(body) = build_tooltip_body(p) else {
        for e in tooltip_q.iter() {
            commands.entity(e).despawn();
        }
        return;
    };

    if tooltip_q.single_mut().is_ok() {
        // Existing tooltip — just refresh its text.
        if let Ok(mut text) = text_q.single_mut()
            && text.0 != body
        {
            text.0 = body;
        }
        return;
    }

    // Spawn fresh tooltip pinned to the bottom-right corner so it
    // never overlaps the centered peek-popup card art.
    let panel = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                bottom: Val::Px(10.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.06, 0.12, 0.93)),
            AltTooltip,
            crate::systems::game_ui::InGameRoot,
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(panel).with_children(|p| {
        p.spawn((
            Text::new(body),
            ui_fonts.tf(13.0),
            TextColor(Color::srgba(0.95, 0.95, 1.0, 1.0)),
            AltTooltipText,
            Pickable::IGNORE,
        ));
    });
}

/// Build the tooltip body. Returns `None` when the card has nothing
/// the peek-popup art doesn't already show — we don't want a tiny
/// dark panel popping up just to repeat "this is a creature with 2/2"
/// while the user is looking at the full card art.
fn build_tooltip_body(p: &crabomination::net::PermanentView) -> Option<String> {
    let mut lines = Vec::new();

    // P/T summary — only if modified, since the peek popup shows the
    // printed P/T as part of the card art.
    if p.card_types.contains(&CardType::Creature)
        && (p.power != p.base_power || p.toughness != p.base_toughness)
    {
        lines.push(format!(
            "{}/{}  (printed {}/{})",
            p.power, p.toughness, p.base_power, p.base_toughness
        ));
    }

    // Creature type-line: surface tribal context ("Inkling Wizard",
    // "Pest", "Spirit Warrior") so the player can see at a glance
    // which tribal anthems / dies-trigger groups this body feeds.
    // Only render when there are creature types to show (filters out
    // colorless artifacts and non-creature permanents). Push
    // (claude/modern_decks batch 198).
    if p.card_types.contains(&CardType::Creature) && !p.creature_types.is_empty() {
        lines.push(format!("Type: {}", p.creature_types.join(" ")));
    }

    // Loyalty for planeswalkers (separate from counters list since it's
    // the headline number on every walker).
    if p.card_types.contains(&CardType::Planeswalker) {
        let loyalty = p
            .counters
            .iter()
            .find_map(|(k, v)| matches!(k, CounterType::Loyalty).then_some(*v))
            .unwrap_or(0);
        lines.push(format!("Loyalty: {loyalty}"));
    }

    // Counters (excluding loyalty, which we already broke out).
    let mut counters: Vec<(CounterType, u32)> = p
        .counters
        .iter()
        .filter(|(k, n)| !matches!(k, CounterType::Loyalty) && *n > 0)
        .map(|(k, n)| (*k, *n))
        .collect();
    counters.sort_by_key(|(k, _)| sort_key(*k));
    if !counters.is_empty() {
        if !lines.is_empty() {
            lines.push(String::from("─────────"));
        }
        for (kind, n) in counters {
            lines.push(format!("{} ×{}", counter_label(kind), n));
        }
    }

    // Effective keywords (after layer effects). Show these so anthem
    // effects ("All your creatures have Lifelink", Inkling Verselord)
    // are visible even when the printed text doesn't include the
    // keyword. Keep the list compact — one line, comma-separated.
    if !p.keywords.is_empty() {
        if !lines.is_empty() {
            lines.push(String::from("─────────"));
        }
        let mut kw_strs: Vec<String> = p.keywords.iter().map(keyword_label).collect();
        kw_strs.sort();
        kw_strs.dedup();
        lines.push(kw_strs.join(", "));
    }

    // Activated abilities — show the cost + effect label so players
    // can see "this Witherbloom Pledgemage has {1}{B}, Pay 1 life:
    // Draw a card" without clicking through to the activator UI.
    let ability_lines: Vec<String> = p
        .abilities
        .iter()
        .filter(|a| !a.is_mana)
        .map(|a| {
            let cost = if a.cost_label.is_empty() { "—".to_string() } else { a.cost_label.clone() };
            format!("{}: {}", cost, a.effect_label)
        })
        .collect();
    if !ability_lines.is_empty() {
        if !lines.is_empty() {
            lines.push(String::from("─────────"));
        }
        for l in ability_lines {
            lines.push(l);
        }
    }

    // Triggered abilities — short event-prefixed labels exposed by the
    // server via `PermanentView.triggered_ability_labels`. Lets players
    // see "ETB: Draw a card", "Magecraft: Drain 1", "Dies: Mill 2"
    // without round-tripping through the card-text panel.
    if !p.triggered_ability_labels.is_empty() {
        if !lines.is_empty() {
            lines.push(String::from("─────────"));
        }
        for l in &p.triggered_ability_labels {
            lines.push(l.clone());
        }
    }

    // Static abilities — printed Oracle wording exposed by the server
    // via `PermanentView.static_ability_labels`. Lets players see
    // "Other Inkling creatures you control get +2/+2." or "Each
    // opponent can't gain life." without scrolling through the card
    // detail panel. Added per the TODO improvement "Static-ability
    // tooltip exposure" — pairs with the existing
    // `triggered_ability_labels` block above.
    if !p.static_ability_labels.is_empty() {
        if !lines.is_empty() {
            lines.push(String::from("─────────"));
        }
        for l in &p.static_ability_labels {
            lines.push(l.clone());
        }
    }

    if p.ward_cost > 0 {
        lines.push(format!("Ward {{{}}}", p.ward_cost));
    }

    // Counter-state highlights — surface the high-signal CR 122.1
    // counter states (shield/finality) that have engine effects
    // beyond their printed +1/+1 cousins. Helps the player see "this
    // creature absorbs one damage/destroy" or "this creature exiles
    // on death" without scrolling the counters list.
    // CR 122.1c shield counters: each absorbs one damage/destroy event.
    // Surface the per-counter count when N > 1 so the player sees how
    // many incoming events the creature can shrug off. Falls back to the
    // legacy boolean badge when the explicit count isn't populated
    // (older server projection / older snapshot).
    if p.shield_counter_count > 1 {
        lines.push(format!(
            "(shielded ×{}: absorbs {} damage/destroy events)",
            p.shield_counter_count, p.shield_counter_count
        ));
    } else if p.has_shield_counters {
        lines.push(String::from("(shielded: next damage/destroy is absorbed)"));
    }
    if p.finality_counter_count > 0 || p.has_finality_counters {
        lines.push(String::from("(finality: exiles instead of going to graveyard)"));
    }
    if p.stun_counter_count > 1 {
        lines.push(format!(
            "(stunned ×{}: next {} untap steps skipped)",
            p.stun_counter_count, p.stun_counter_count
        ));
    } else if p.has_stun_counters {
        lines.push(String::from("(stunned: next untap is skipped)"));
    }
    // Surface +1/+1 and -1/-1 counter highlights — the most common
    // counter shapes carry a P/T delta that's often more important than
    // the printed body. Push (modern_decks batch 174): added the
    // has_plus_one_counters / has_minus_one_counters helpers on
    // PermanentView so the client doesn't have to scan the `counters`
    // vec; surface them here.
    // Read the explicit counts off the `counters` vec so the badge can
    // show the actual P/T swing (e.g. "(boosted: +3/+3 from 3 +1/+1
    // counters)") — far more useful than a bare boolean when an enrage
    // creature or a Quandrix Fractal has stacked several counters. Falls
    // back to the legacy boolean badge if the explicit vec is empty but
    // the helper flag is set (older server projection / snapshot).
    let plus_n = p
        .counters
        .iter()
        .find_map(|(k, n)| matches!(k, CounterType::PlusOnePlusOne).then_some(*n))
        .unwrap_or(0);
    let minus_n = p
        .counters
        .iter()
        .find_map(|(k, n)| matches!(k, CounterType::MinusOneMinusOne).then_some(*n))
        .unwrap_or(0);
    if plus_n > 0 {
        lines.push(format!(
            "(boosted: +{plus_n}/+{plus_n} from {plus_n} +1/+1 counter{})",
            if plus_n == 1 { "" } else { "s" }
        ));
    } else if p.has_plus_one_counters {
        lines.push(String::from("(boosted: +1/+1 counters)"));
    }
    if minus_n > 0 {
        lines.push(format!(
            "(weakened: -{minus_n}/-{minus_n} from {minus_n} -1/-1 counter{})",
            if minus_n == 1 { "" } else { "s" }
        ));
    } else if p.has_minus_one_counters {
        lines.push(String::from("(weakened: -1/-1 counters)"));
    }
    // Surface CR 122.1b keyword counters — one line per active counter
    // type (flying, first strike, deathtouch, trample, lifelink, haste,
    // vigilance, reach). Push (modern_decks batch 187): added.
    for (kw, n) in &p.keyword_counters {
        let label = format!("{:?}", kw);
        if *n == 1 {
            lines.push(format!("({} counter granting {})", label.to_lowercase(), label));
        } else {
            lines.push(format!("({n} {} counters)", label.to_lowercase()));
        }
    }

    if p.tapped {
        lines.push(String::from("(tapped)"));
    }

    // Combat status: surface "(attacking)" / "(blocking attacker N)"
    // so the player can tell at a glance which creatures are committed
    // to combat. Push (claude/modern_decks batch 202).
    if p.attacking {
        lines.push(String::from("(attacking)"));
    }
    if let Some(att) = p.blocking_attacker {
        lines.push(format!("(blocking #{})", att.0));
    }

    // Marked damage: every creature with non-zero damage is one toughness-
    // threshold away from death. Surface "marked: N damage" plus a
    // (lethal? Y/N) shorthand so the player sees at a glance how close
    // the creature is to dying. Push (claude/modern_decks batch 162) —
    // covers CR 121-style damage tracking. Hidden when no damage marked
    // (the common case for fresh permanents).
    if p.damage > 0 && p.card_types.contains(&CardType::Creature) {
        if p.damage as i32 >= p.toughness {
            lines.push(format!("(marked: {} damage — LETHAL)", p.damage));
        } else {
            // Surface the survival margin so the player sees how much
            // more damage kills the creature — pairs with the combat
            // status lines for at-a-glance combat math.
            let to_live = p.toughness - p.damage as i32;
            lines.push(format!(
                "(marked: {} damage; {} more lethal)",
                p.damage, to_live
            ));
        }
    }

    // Summoning sickness: creatures that entered this turn can't attack
    // or use {T} activated abilities (per CR 302.1). Show this in the
    // tooltip so players don't accidentally tap a fresh creature
    // expecting an attack.
    if p.summoning_sick && p.card_types.contains(&CardType::Creature) {
        lines.push(String::from("(summoning sick)"));
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}


/// Render a `Keyword` as a short human string for the tooltip. Keeps
/// the labels short ("Lifelink", "First Strike") so a card with several
/// granted keywords doesn't blow out the tooltip line.
fn keyword_label(kw: &crabomination::card::Keyword) -> String {
    use crabomination::card::Keyword as K;
    use crabomination::mana::Color;
    let color_word = |c: &Color| -> &'static str {
        match c {
            Color::White => "white",
            Color::Blue => "blue",
            Color::Black => "black",
            Color::Red => "red",
            Color::Green => "green",
        }
    };
    match kw {
        K::Flying => "Flying".into(),
        K::FirstStrike => "First Strike".into(),
        K::DoubleStrike => "Double Strike".into(),
        K::Lifelink => "Lifelink".into(),
        K::Vigilance => "Vigilance".into(),
        K::Trample => "Trample".into(),
        K::Deathtouch => "Deathtouch".into(),
        K::Haste => "Haste".into(),
        K::Menace => "Menace".into(),
        K::Reach => "Reach".into(),
        K::Defender => "Defender".into(),
        K::Indestructible => "Indestructible".into(),
        K::Hexproof => "Hexproof".into(),
        K::Flash => "Flash".into(),
        K::Shroud => "Shroud".into(),
        // Surface Ward's cost as "Ward {2}" or "Ward—pay 2 life"
        // instead of the prior `{:?}` shape that printed the raw
        // enum variant text.
        K::Ward(wc) => match wc {
            crabomination::card::WardCost::Mana(c) => format!("Ward {}", c.summary()),
            crabomination::card::WardCost::Life(n) => format!("Ward—Pay {n} life"),
            crabomination::card::WardCost::Discard(n) => format!("Ward—Discard {n}"),
            crabomination::card::WardCost::SacrificeCreature => "Ward—Sacrifice a creature".into(),
        },
        // Protection rolls up the color name in lowercase to match
        // printed Oracle ("protection from white", not "from White").
        K::Protection(c) => format!("Protection from {}", color_word(c)),
        // Cycling / Flashback should expose their cost so the activator
        // can see what they'd pay.
        K::Cycling(cost) => format!("Cycling {}", cost.summary()),
        K::Flashback(cost) => format!("Flashback {}", cost.summary()),
        K::Convoke => "Convoke".into(),
        K::Persist => "Persist".into(),
        K::Undying => "Undying".into(),
        K::CantBeCountered => "Can't be countered".into(),
        _ => format!("{kw:?}"),
    }
}

fn sort_key(kind: CounterType) -> u8 {
    match kind {
        CounterType::PlusOnePlusOne => 0,
        CounterType::MinusOneMinusOne => 1,
        CounterType::Charge => 2,
        CounterType::Stun => 3,
        CounterType::Time => 4,
        CounterType::Poison => 5,
        CounterType::Energy => 6,
        _ => 7,
    }
}

fn counter_label(kind: CounterType) -> &'static str {
    match kind {
        CounterType::PlusOnePlusOne => "+1/+1",
        CounterType::MinusOneMinusOne => "-1/-1",
        CounterType::Loyalty => "Loyalty",
        CounterType::Charge => "Charge",
        CounterType::Stun => "Stun",
        CounterType::Time => "Time",
        CounterType::Poison => "Poison",
        CounterType::Lore => "Lore",
        CounterType::Fade => "Fade",
        CounterType::Age => "Age",
        CounterType::Level => "Level",
        CounterType::Energy => "Energy",
        CounterType::Experience => "Experience",
        CounterType::Verse => "Verse",
        CounterType::Shield => "Shield",
        CounterType::Wish => "Wish",
        CounterType::Page => "Page",
        CounterType::Growth => "Growth",
        CounterType::Prepared => "Prepared",
        CounterType::Finality => "Finality",
    }
}

#[cfg(test)]
mod tests {
    use super::build_tooltip_body;
    use crabomination::card::{CardId, CardType, CounterType};
    use crabomination::net::PermanentView;

    fn make_permanent_view(damage: u32, toughness: i32) -> PermanentView {
        PermanentView {
            id: CardId(0),
            name: "Grizzly Bears".into(),
            controller: 0,
            owner: 0,
            card_types: vec![CardType::Creature],
            tapped: false,
            damage,
            summoning_sick: false,
            power: 2,
            toughness,
            base_power: 2,
            base_toughness: 2,
            keywords: vec![],
            counters: vec![],
            attached_to: None,
            is_token: false,
            attacking: false,
            blocking_attacker: None,
            triggered_ability_labels: vec![],
            static_ability_labels: vec![],
            abilities: vec![],
            loyalty_abilities: vec![],
            has_stun_counters: false,
            has_finality_counters: false,
            has_shield_counters: false,
            pt_modified: false,
            mana_cost_display: String::new(),
            creature_types: vec![],
            ward_cost: 0,
            mana_value: 0,
            is_legendary: false,
            has_plus_one_counters: false,
            has_minus_one_counters: false,
            total_counter_count: 0,
            keyword_counters: vec![],
            shield_counter_count: 0,
            stun_counter_count: 0,
            finality_counter_count: 0,
        }
    }

    #[test]
    fn marked_damage_shows_when_creature_has_damage() {
        let p = make_permanent_view(1, 2);
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("marked: 1 damage"), "got: {body}");
        assert!(!body.contains("LETHAL"), "1 damage on a 2-tough body isn't lethal: {body}");
    }

    #[test]
    fn marked_damage_shows_survival_margin_when_not_lethal() {
        // 2 damage on a 5-toughness body → 3 more is lethal.
        let p = make_permanent_view(2, 5);
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("marked: 2 damage; 3 more lethal"), "got: {body}");
        assert!(!body.contains("LETHAL"), "not lethal yet: {body}");
    }

    #[test]
    fn marked_damage_calls_out_lethal_when_equal_or_greater_than_toughness() {
        let p = make_permanent_view(2, 2);
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("marked: 2 damage"), "got: {body}");
        assert!(body.contains("LETHAL"),
            "2 damage on a 2-tough body should be flagged lethal: {body}");
    }

    #[test]
    fn marked_damage_hidden_when_zero() {
        let p = make_permanent_view(0, 2);
        // No counters, no abilities, no other lines — body might be None.
        let body = build_tooltip_body(&p);
        if let Some(s) = body {
            assert!(!s.contains("marked:"), "no damage marked, should not surface: {s}");
        }
    }

    #[test]
    fn creature_type_line_renders_when_creature_types_present() {
        let mut p = make_permanent_view(0, 2);
        p.creature_types = vec!["Bear".into()];
        let body = build_tooltip_body(&p).expect("body should render");
        assert!(body.contains("Type: Bear"), "got: {body}");
    }

    #[test]
    fn creature_type_line_hidden_for_non_creature_even_when_subtypes_set() {
        let mut p = make_permanent_view(0, 2);
        p.card_types = vec![CardType::Enchantment];
        p.creature_types = vec!["Bear".into()];
        let body = build_tooltip_body(&p);
        if let Some(s) = body {
            assert!(!s.contains("Type:"), "non-creature should not show Type: line, got: {s}");
        }
    }

    #[test]
    fn marked_damage_unused_for_non_creature() {
        let mut p = make_permanent_view(3, 2);
        // Re-shape as an enchantment — damage on non-creatures is bogus,
        // but if we ever stamp it (engine bug), the tooltip should hide it.
        p.card_types = vec![CardType::Enchantment];
        let body = build_tooltip_body(&p);
        if let Some(s) = body {
            assert!(!s.contains("marked:"), "non-creature should never surface damage: {s}");
        }
    }

    #[test]
    fn keyword_counter_surfaces_in_tooltip() {
        use crabomination::card::Keyword;
        let mut p = make_permanent_view(0, 2);
        p.keyword_counters = vec![(Keyword::Flying, 1)];
        let body = build_tooltip_body(&p).expect("tooltip should render");
        // Surface a flying counter.
        assert!(body.to_lowercase().contains("flying"), "got: {body}");
    }

    #[test]
    fn shield_counter_count_renders_in_tooltip() {
        let mut p = make_permanent_view(0, 2);
        p.shield_counter_count = 3;
        p.has_shield_counters = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        // Should show the explicit count, not just the boolean.
        assert!(body.contains("shielded ×3"), "got: {body}");
    }

    #[test]
    fn shield_counter_single_falls_back_to_boolean_message() {
        let mut p = make_permanent_view(0, 2);
        p.shield_counter_count = 1;
        p.has_shield_counters = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.to_lowercase().contains("shielded"), "got: {body}");
        assert!(!body.contains("shielded ×"), "single shield uses boolean form: {body}");
    }

    #[test]
    fn stun_counter_count_renders_in_tooltip() {
        let mut p = make_permanent_view(0, 2);
        p.stun_counter_count = 2;
        p.has_stun_counters = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("stunned ×2"), "got: {body}");
    }

    #[test]
    fn no_keyword_counter_no_keyword_line() {
        let p = make_permanent_view(0, 2);
        let body = build_tooltip_body(&p);
        if let Some(s) = body {
            assert!(!s.to_lowercase().contains("counter granting"),
                "no keyword counters: {s}");
        }
    }

    // Silence unused-import warnings for items only used in some tests.
    #[allow(dead_code)]
    fn _ensure_counter_type_import_used() {
        let _ = CounterType::PlusOnePlusOne;
    }

    #[test]
    fn attacking_status_surfaces_in_tooltip() {
        let mut p = make_permanent_view(0, 2);
        p.attacking = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("(attacking)"), "got: {body}");
    }

    #[test]
    fn blocking_status_shows_attacker_id() {
        let mut p = make_permanent_view(0, 2);
        p.blocking_attacker = Some(CardId(7));
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("(blocking #7)"), "got: {body}");
    }

    #[test]
    fn combat_status_hidden_when_idle() {
        let p = make_permanent_view(0, 2);
        let body = build_tooltip_body(&p);
        if let Some(s) = body {
            assert!(!s.contains("(attacking)"), "no attack flag: {s}");
            assert!(!s.contains("(blocking"), "no block flag: {s}");
        }
    }

    #[test]
    fn plus_one_counters_show_numeric_pt_delta() {
        // Push (claude/modern_decks batch 205): the +1/+1 badge now shows
        // the actual swing read off the counters vec — useful for enrage
        // creatures that have stacked several counters.
        let mut p = make_permanent_view(0, 4);
        p.counters = vec![(CounterType::PlusOnePlusOne, 3)];
        p.has_plus_one_counters = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("(boosted: +3/+3 from 3 +1/+1 counters)"),
            "expected numeric +1/+1 badge: {body}");
    }

    #[test]
    fn single_plus_one_counter_uses_singular_wording() {
        let mut p = make_permanent_view(0, 3);
        p.counters = vec![(CounterType::PlusOnePlusOne, 1)];
        p.has_plus_one_counters = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("(boosted: +1/+1 from 1 +1/+1 counter)"),
            "expected singular wording: {body}");
        assert!(!body.contains("counters)"), "no plural 's' on a single counter: {body}");
    }

    #[test]
    fn minus_one_counters_show_numeric_pt_delta() {
        let mut p = make_permanent_view(0, 5);
        p.counters = vec![(CounterType::MinusOneMinusOne, 2)];
        p.has_minus_one_counters = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("(weakened: -2/-2 from 2 -1/-1 counters)"),
            "expected numeric -1/-1 badge: {body}");
    }

    #[test]
    fn plus_one_badge_falls_back_to_boolean_without_explicit_count() {
        // Older server projection: helper flag set but counters vec empty.
        let mut p = make_permanent_view(0, 3);
        p.has_plus_one_counters = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("(boosted: +1/+1 counters)"),
            "expected legacy boolean badge fallback: {body}");
    }
}
