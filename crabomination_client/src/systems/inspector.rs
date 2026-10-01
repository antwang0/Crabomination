//! The card inspector: what a hovered permanent *is now*, under its printed
//! text in the hover preview (`ui::hover_card_preview`).
//!
//! The preview shows the printed card, and cards render art-only, so the
//! board's own state — a pumped body, a keyword an anthem granted, the
//! counters and attachments it carries, who controls it, what it's doing in
//! combat — sat on chips and coins, or behind Alt (`counter_tooltip`'s full
//! list in the corner). The inspector reads it off the view and says it in
//! words beside the card: what changed from the printed card and why, then,
//! for a permanent the viewer can use, what they can do with it right now —
//! the activated abilities the engine would accept
//! (`ClientView.activatable_abilities`, probed like any other action),
//! attacking, blocking, turning it face up.

use bevy::prelude::Color;
use crabomination::card::{CardDefinition, CardType, CounterType, Keyword};
use crabomination::game::AttackTarget;
use crabomination::net::{ClientView, PermanentView};

use crate::systems::counter_tooltip::{counter_label, keyword_label, sort_key};
use crate::systems::game_ui::table_awareness::seat_label;
use crate::theme;

/// How an inspector line reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// A fact: who controls it, what it's attacking.
    Plain,
    /// Better than printed: a bigger body, a granted keyword.
    Up,
    /// Worse: shrunk, a keyword lost, damage marked, locked down.
    Down,
    /// Something the viewer can do with it now.
    Ready,
    /// A state worth knowing that isn't a change: tapped, summoning sick.
    Muted,
}

impl Tone {
    pub fn color(self) -> Color {
        match self {
            Tone::Plain => theme::TEXT_BODY,
            Tone::Up => theme::TEXT_GOOD,
            Tone::Down => theme::TEXT_DANGER,
            Tone::Ready => theme::TEXT_INFO,
            Tone::Muted => theme::TEXT_MUTED,
        }
    }
}

/// One line of the inspector.
#[derive(Clone, Debug, PartialEq)]
pub struct NowLine {
    pub text: String,
    pub tone: Tone,
}

fn line(text: impl Into<String>, tone: Tone) -> NowLine {
    NowLine { text: text.into(), tone }
}

/// The printed face `name` names: the back face of a double-faced card when
/// that is the face shown.
pub(crate) fn printed_face(name: &str) -> Option<CardDefinition> {
    let mut def = crabomination::catalog::lookup_by_name(name)?;
    if def.back_face.as_ref().is_some_and(|back| back.name == name)
        && let Some(back) = def.back_face.take()
    {
        def = *back;
    }
    Some(def)
}

/// What the permanent `pv` is now, then what the viewer can do with it.
/// Empty for a permanent that is just as printed and offers nothing.
pub fn now_lines(cv: &ClientView, pv: &PermanentView) -> Vec<NowLine> {
    let printed = if pv.face_down { None } else { printed_face(&pv.name) };
    let mut out = Vec::new();
    body(&mut out, pv);
    keywords(&mut out, pv, printed.as_ref());
    out.extend(override_lines(printed.as_ref(), pv.lost_all_abilities, &pv.creature_subtypes, &pv.colors));
    counters(&mut out, pv);
    attachments(&mut out, cv, pv);
    status(&mut out, cv, pv);
    actions(&mut out, cv, pv);
    out
}

/// The P/T a counter of `kind` adds (CR 613.4c), as the layer pass reads it.
fn counter_pt(kind: CounterType) -> (i32, i32) {
    match kind {
        CounterType::PlusOnePlusOne => (1, 1),
        CounterType::MinusOneMinusOne => (-1, -1),
        CounterType::MinusZeroMinusOne => (0, -1),
        CounterType::MinusZeroMinusTwo => (0, -2),
        CounterType::MinusOneMinusZero => (-1, 0),
        CounterType::PlusOnePlusZero => (1, 0),
        CounterType::PlusZeroPlusOne => (0, 1),
        CounterType::PlusTwoPlusZero => (2, 0),
        CounterType::PlusZeroPlusTwo => (0, 2),
        CounterType::PlusTwoPlusTwo => (2, 2),
        _ => (0, 0),
    }
}

/// A creature's body against its printed one, split into what its counters
/// add and what every other effect does; then the damage marked on it.
fn body(out: &mut Vec<NowLine>, pv: &PermanentView) {
    if pv.face_down {
        let text = match &pv.face_down_name {
            Some(name) => format!("Face-down 2/2 (it's {name})"),
            None => "Face-down 2/2".to_string(),
        };
        out.push(line(text, Tone::Plain));
    }
    if !pv.is_creature() {
        return;
    }
    let delta = (pv.power - pv.base_power, pv.toughness - pv.base_toughness);
    if delta != (0, 0) {
        let from_counters = pv.counters.iter().fold((0, 0), |(p, t), (kind, n)| {
            let (dp, dt) = counter_pt(*kind);
            (p + dp * *n as i32, t + dt * *n as i32)
        });
        let from_effects = (delta.0 - from_counters.0, delta.1 - from_counters.1);
        let signed = |(p, t): (i32, i32)| format!("{p:+}/{t:+}");
        let mut why = Vec::new();
        if from_counters != (0, 0) {
            why.push(format!("{} from counters", signed(from_counters)));
        }
        if from_effects != (0, 0) {
            why.push(format!("{} from effects", signed(from_effects)));
        }
        let tone = match delta {
            (p, t) if p >= 0 && t >= 0 => Tone::Up,
            (p, t) if p <= 0 && t <= 0 => Tone::Down,
            _ => Tone::Plain,
        };
        let text = format!(
            "{}/{} (printed {}/{}): {}",
            pv.power, pv.toughness, pv.base_power, pv.base_toughness, why.join(", "),
        );
        out.push(line(text, tone));
    }
    if pv.damage > 0 {
        let left = pv.toughness - pv.damage as i32;
        let text = if left <= 0 || pv.marked_lethal {
            format!("{} damage marked: lethal", pv.damage)
        } else {
            format!("{} damage marked: {left} more is lethal", pv.damage)
        };
        out.push(line(text, Tone::Down));
    }
}

/// Keywords it has that the printed card doesn't (an anthem's, an Aura's, a
/// keyword counter's) and printed ones it has lost. A token or a face-down
/// permanent has no printed card to hold them against.
fn keywords(out: &mut Vec<NowLine>, pv: &PermanentView, printed: Option<&CardDefinition>) {
    let Some(def) = printed else { return };
    let labels = |kws: Vec<&Keyword>| {
        let mut labels: Vec<String> = kws.into_iter().map(keyword_label).collect();
        labels.sort();
        labels.dedup();
        labels.join(", ")
    };
    let gained: Vec<&Keyword> = pv.keywords.iter().filter(|k| !def.keywords.contains(k)).collect();
    if !gained.is_empty() {
        out.push(line(format!("Has {}", labels(gained)), Tone::Up));
    }
    // Losing every ability is said once, by the override line.
    let lost: Vec<&Keyword> = def.keywords.iter().filter(|k| !pv.keywords.contains(k)).collect();
    if !lost.is_empty() && !pv.lost_all_abilities {
        out.push(line(format!("Lost {}", labels(lost)), Tone::Down));
    }
}

/// Characteristics a continuous effect has overwritten (Ichthyomorphosis,
/// Heliod's Punishment, Turn to Frog): every ability gone, creature types or
/// colours other than the printed card's. Quiet when nothing diverges.
pub(crate) fn override_lines(
    printed: Option<&CardDefinition>,
    lost_all_abilities: bool,
    creature_subtypes: &[crabomination::card::CreatureType],
    colors: &[crabomination::mana::Color],
) -> Vec<NowLine> {
    let mut out = Vec::new();
    if lost_all_abilities {
        out.push(line("Lost all abilities", Tone::Down));
    }
    let Some(def) = printed else { return out };
    // Only an actual change, so vanilla creatures stay quiet.
    if !creature_subtypes.is_empty() && creature_subtypes != def.subtypes.creature_types.as_slice() {
        let types: Vec<String> = creature_subtypes.iter().map(|t| format!("{t:?}")).collect();
        out.push(line(format!("Now a {}", types.join(" ")), Tone::Down));
    }
    let mut printed_colors = def.cost.colors();
    for c in &def.color_indicator {
        if !printed_colors.contains(c) {
            printed_colors.push(*c);
        }
    }
    if colors != printed_colors.as_slice() {
        let label = if colors.is_empty() {
            "colorless".to_string()
        } else {
            colors.iter().map(|c| format!("{c:?}").to_lowercase()).collect::<Vec<_>>().join(" and ")
        };
        out.push(line(format!("Now {label}"), Tone::Down));
    }
    out
}

/// Its counters, loyalty first on a planeswalker; the P/T line above already
/// says what the +1/+1 kind adds up to.
fn counters(out: &mut Vec<NowLine>, pv: &PermanentView) {
    if pv.card_types.contains(&CardType::Planeswalker) {
        let loyalty = pv.counters.iter().find(|(k, _)| *k == CounterType::Loyalty).map_or(0, |(_, n)| *n);
        let text = match pv.loyalty_uses_remaining {
            Some(0) => format!("Loyalty {loyalty}: no activations left this turn"),
            _ => format!("Loyalty {loyalty}"),
        };
        out.push(line(text, Tone::Plain));
    }
    let mut kinds: Vec<(CounterType, u32)> =
        pv.counters.iter().filter(|(k, n)| *k != CounterType::Loyalty && *n > 0).copied().collect();
    if kinds.is_empty() {
        return;
    }
    kinds.sort_by_key(|(k, _)| sort_key(*k));
    let list: Vec<String> = kinds.iter().map(|(k, n)| format!("{} ×{n}", counter_label(*k))).collect();
    out.push(line(format!("Counters: {}", list.join(", ")), Tone::Plain));
}

/// What is attached to it, by kind, and what it is attached to.
fn attachments(out: &mut Vec<NowLine>, cv: &ClientView, pv: &PermanentView) {
    let on_it: Vec<&PermanentView> = cv.battlefield.iter().filter(|q| q.attached_to == Some(pv.id)).collect();
    let named = |pick: &dyn Fn(&PermanentView) -> bool| -> Option<String> {
        let names: Vec<&str> = on_it.iter().filter(|q| pick(q)).map(|q| q.name.as_str()).collect();
        (!names.is_empty()).then(|| names.join(", "))
    };
    let is_aura = |q: &PermanentView| q.card_types.contains(&CardType::Enchantment);
    let is_equipment = |q: &PermanentView| !is_aura(q) && q.card_types.contains(&CardType::Artifact);
    if let Some(names) = named(&is_equipment) {
        out.push(line(format!("Equipped with {names}"), Tone::Plain));
    }
    if let Some(names) = named(&is_aura) {
        out.push(line(format!("Enchanted by {names}"), Tone::Plain));
    }
    if let Some(names) = named(&|q| !is_aura(q) && !is_equipment(q)) {
        out.push(line(format!("Attached: {names}"), Tone::Plain));
    }
    let verb = if is_aura(pv) { "Enchanting" } else if is_equipment(pv) { "Equipping" } else { "Attached to" };
    if let Some(host) = &pv.attached_to_name {
        out.push(line(format!("{verb} {host}"), Tone::Plain));
    } else if let Some(seat) = pv.attached_to_player {
        out.push(line(format!("{verb} {}", seat_label(&cv.players, cv.your_seat, seat)), Tone::Plain));
    }
}

/// Who controls it, and what it's doing or kept from doing this turn.
fn status(out: &mut Vec<NowLine>, cv: &ClientView, pv: &PermanentView) {
    let seat = |s: usize| seat_label(&cv.players, cv.your_seat, s);
    let name_of = |id| cv.battlefield.iter().find(|c| c.id == id).map_or_else(|| "a permanent".into(), |c| c.name.clone());
    if pv.controller != pv.owner {
        out.push(line(format!("Controlled by {}, owned by {}", seat(pv.controller), seat(pv.owner)), Tone::Plain));
    }
    if pv.attacking {
        let text = match pv.attack_target {
            Some(AttackTarget::Player(s)) => format!("Attacking {}", seat(s)),
            Some(AttackTarget::Planeswalker(id) | AttackTarget::Battle(id)) => format!("Attacking {}", name_of(id)),
            None => "Attacking".to_string(),
        };
        out.push(line(text, Tone::Plain));
    }
    if !pv.blocking_attackers.is_empty() {
        let names: Vec<String> = pv.blocking_attackers.iter().map(|id| name_of(*id)).collect();
        out.push(line(format!("Blocking {}", names.join(", ")), Tone::Plain));
    }
    if pv.tapped {
        out.push(line("Tapped", Tone::Muted));
    }
    // Sickness only binds on its controller's turn (CR 302.6): an
    // opponent's creature that came in on yours can still block.
    if pv.summoning_sick && pv.is_creature() && cv.active_player == pv.controller && !pv.keywords.contains(&Keyword::Haste) {
        out.push(line("Summoning sick: can't attack or {T} this turn", Tone::Muted));
    }
    if !pv.goaded_by.is_empty() {
        let names: Vec<String> = pv.goaded_by.iter().map(|s| seat(*s)).collect();
        out.push(line(format!("Goaded by {}: attacks each combat, not them if able", names.join(", ")), Tone::Down));
    }
    if pv.cant_attack_this_turn {
        out.push(line("Can't attack this turn", Tone::Down));
    }
    if pv.detained {
        out.push(line("Detained: can't attack, block or activate", Tone::Down));
    }
    if pv.wont_untap || pv.untap_locked {
        out.push(line("Won't untap in its next untap step", Tone::Down));
    }
}

/// What the viewer can do with it now, each checked by the engine: the
/// abilities `activatable_abilities` and `activatable_loyalty` name,
/// attacking (`legal_attackers`), blocking (`legal_block_targets`) and
/// turning it face up.
fn actions(out: &mut Vec<NowLine>, cv: &ClientView, pv: &PermanentView) {
    for (_, index) in cv.activatable_abilities.iter().filter(|(id, _)| *id == pv.id) {
        let Some(ability) = pv.abilities.iter().find(|a| a.index == *index) else { continue };
        let cost = if ability.cost_label.is_empty() { "{0}" } else { ability.cost_label.as_str() };
        out.push(line(format!("▶ {cost}: {}", ability.effect_label), Tone::Ready));
    }
    for (_, index) in cv.activatable_loyalty.iter().filter(|(id, _)| *id == pv.id) {
        let Some(ability) = pv.loyalty_abilities.iter().find(|a| a.index == *index) else { continue };
        // As printed: +2, 0, −1, −X.
        let cost = match ability.loyalty_cost {
            _ if ability.x_cost => "−X".to_string(),
            0 => "0".to_string(),
            c if c < 0 => format!("−{}", -c),
            c => format!("+{c}"),
        };
        out.push(line(format!("▶ {cost}: {}", ability.effect_label), Tone::Ready));
    }
    if cv.turn_up_able.contains(&pv.id) {
        out.push(line("▶ Can be turned face up", Tone::Ready));
    }
    if cv.legal_attackers.contains(&pv.id) {
        out.push(line("▶ Can attack", Tone::Ready));
    }
    if cv.legal_blockers.contains(&pv.id) {
        let names: Vec<String> = cv
            .legal_block_targets
            .iter()
            .find(|(blocker, _)| *blocker == pv.id)
            .map(|(_, attackers)| {
                attackers.iter().filter_map(|a| cv.battlefield.iter().find(|c| c.id == *a)).map(|c| c.name.clone()).collect()
            })
            .unwrap_or_default();
        let text = if names.is_empty() { "▶ Can block".to_string() } else { format!("▶ Can block {}", names.join(", ")) };
        out.push(line(text, Tone::Ready));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crabomination::catalog;
    use crabomination::game::{GameState, TurnStep};

    fn view(g: &GameState) -> ClientView {
        crabomination::server::view::project(g, 0)
    }

    fn lines_for(g: &GameState, id: crabomination::card::CardId) -> Vec<NowLine> {
        let cv = view(g);
        let pv = cv.battlefield.iter().find(|p| p.id == id).expect("on the battlefield").clone();
        now_lines(&cv, &pv)
    }

    fn texts(lines: &[NowLine]) -> Vec<&str> {
        lines.iter().map(|l| l.text.as_str()).collect()
    }

    /// A creature just as printed, doing nothing, says nothing.
    #[test]
    fn a_creature_as_printed_says_nothing() {
        let mut g = crabomination::game::two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(bear);
        assert_eq!(lines_for(&g, bear), vec![]);
    }

    /// A pumped body says how far from printed it is and why: its counters,
    /// and the rest from effects (an anthem, an Aura). The Aura's keyword
    /// shows as gained, the Aura itself as what enchants it.
    #[test]
    fn a_pumped_creature_says_why() {
        let mut g = crabomination::game::two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        g.battlefield_find_mut(bear).unwrap().counters.insert(CounterType::PlusOnePlusOne, 1);
        g.add_card_to_battlefield(0, catalog::glorious_anthem());
        let rancor = g.add_card_to_battlefield(0, catalog::rancor());
        g.battlefield_find_mut(rancor).unwrap().attached_to = Some(bear);
        g.battlefield_find_mut(bear).unwrap().damage = 1;

        let lines = lines_for(&g, bear);
        assert_eq!(lines[0], line("6/4 (printed 2/2): +1/+1 from counters, +3/+1 from effects", Tone::Up));
        let said = texts(&lines);
        assert!(said.contains(&"Has Trample"), "{said:?}");
        assert!(said.contains(&"Counters: +1/+1 ×1"), "{said:?}");
        assert!(said.contains(&"Enchanted by Rancor"), "{said:?}");
        assert!(said.contains(&"1 damage marked: 3 more is lethal"), "{said:?}");
        let aura = lines_for(&g, rancor);
        assert_eq!(texts(&aura), ["Enchanting Grizzly Bears"]);
    }

    /// Of a permanent's abilities, the ones the engine would accept now are
    /// offered — Walking Ballista with a counter and no mana can ping, not
    /// grow — and only while the viewer holds priority.
    #[test]
    fn the_live_abilities_are_offered() {
        let mut g = crabomination::game::two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let ballista = g.add_card_to_battlefield(0, catalog::walking_ballista());
        g.clear_sickness(ballista);
        g.battlefield_find_mut(ballista).unwrap().counters.insert(CounterType::PlusOnePlusOne, 1);

        let ready: Vec<NowLine> = lines_for(&g, ballista).into_iter().filter(|l| l.tone == Tone::Ready).collect();
        assert_eq!(ready.len(), 1, "{ready:?}");
        assert!(ready[0].text.starts_with("▶ ") && !ready[0].text.contains("{4}"), "{ready:?}");

        g.priority.player_with_priority = 1;
        assert!(lines_for(&g, ballista).iter().all(|l| l.tone != Tone::Ready), "off priority");
    }

    /// In the declare-attackers step a creature that can attack says so; a
    /// creature that came in this turn says it's summoning sick instead —
    /// on its controller's turn only.
    #[test]
    fn attacking_and_sickness() {
        let mut g = crabomination::game::two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        let ready = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(ready);
        let fresh = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());

        assert_eq!(lines_for(&g, ready), vec![line("▶ Can attack", Tone::Ready)]);
        assert_eq!(texts(&lines_for(&g, fresh)), ["Summoning sick: can't attack or {T} this turn"]);
        assert_eq!(lines_for(&g, theirs), vec![], "sickness doesn't bind on your turn");
    }

    /// A planeswalker offers the loyalty abilities its loyalty pays for, in
    /// its controller's main phase: Liliana of the Veil at 3, +1 and −2.
    #[test]
    fn the_live_loyalty_abilities_are_offered() {
        let mut g = crabomination::game::two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let lili = g.add_card_to_battlefield(0, catalog::liliana_of_the_veil());
        g.battlefield_find_mut(lili).unwrap().counters.insert(CounterType::Loyalty, 3);

        let lines = lines_for(&g, lili);
        assert_eq!(lines[0], line("Loyalty 3", Tone::Plain));
        let ready: Vec<&str> = lines.iter().filter(|l| l.tone == Tone::Ready).map(|l| l.text.as_str()).collect();
        assert_eq!(ready.len(), 2, "{ready:?}");
        assert!(ready[0].starts_with("▶ +1: ") && ready[1].starts_with("▶ −2: "), "{ready:?}");
    }

    /// A stolen permanent names who controls it and who owns it.
    #[test]
    fn control_and_ownership() {
        let mut g = crabomination::game::two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(bear);
        g.battlefield_find_mut(bear).unwrap().controller = 0;
        let cv = view(&g);
        let owner = seat_label(&cv.players, 0, 1);
        let lines = lines_for(&g, bear);
        assert_eq!(texts(&lines), [format!("Controlled by You, owned by {owner}").as_str()]);
    }

    /// Serra Angel turned into a blue Fish with no abilities
    /// (Ichthyomorphosis) says all three; the lost keywords aren't listed
    /// again under "lost all abilities".
    #[test]
    fn override_lines_note_fish_and_lost_abilities() {
        let angel = printed_face("Serra Angel");
        let lines = override_lines(
            angel.as_ref(),
            true,
            &[crabomination::card::CreatureType::Fish],
            &[crabomination::mana::Color::Blue],
        );
        assert_eq!(texts(&lines), ["Lost all abilities", "Now a Fish", "Now blue"]);
    }

    #[test]
    fn override_lines_quiet_for_unchanged_creature() {
        // A Grizzly Bears still a Bear with its abilities → no override notes.
        let bears = printed_face("Grizzly Bears");
        let lines = override_lines(
            bears.as_ref(),
            false,
            &[crabomination::card::CreatureType::Bear],
            &[crabomination::mana::Color::Green],
        );
        assert!(lines.is_empty(), "no notes when nothing diverges");
    }
}
