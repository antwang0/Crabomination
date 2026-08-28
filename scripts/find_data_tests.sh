#!/usr/bin/env bash
# Find pure-data tests: tests that only read CardDefinition fields without
# any GameState interaction. Outputs `file:line fn_name` per candidate, with
# a leading `[CR]` when the doc comment above the test cites a rule, a ruling,
# a bug fix or a regression — those are **sacred** and stay however redundant
# they look. Everything else is a candidate to delete or to fold into one
# table-driven definition audit per set.
#
# A pure-data test:
#  - contains `let _ = catalog::*()` or `catalog::*()` calls
#  - does NOT contain any GameState-touching identifier
#
# **This script was wrong FIVE ways until 2026-08-28** and its output is a
# DELETE LIST, so read the fixes before trusting an older count.
#
#  1. The `fn foo() {` line's opening brace was never counted, so
#     `brace_depth` was 0 on the first body line and every test "ended" after
#     ONE line: a test whose first line read a definition was reported however
#     much engine it touched below (47 such), and a pure-data test whose first
#     line was anything else was missed (167 such).
#  2. It scanned only the test body, so a test that delegates to a local
#     helper looked pure-data whatever the helper does. `assert_school_land`
#     in `sos/hybrid_lands.rs` builds a game and activates two mana abilities;
#     its six callers were on the list. The scan is two passes now and splices
#     same-file helper bodies in (twice, so a helper calling a helper is
#     covered).
#  3. Sacredness read only the doc comment above the test, so a CR citation on
#     the assert itself did not protect it —
#     `magmablood_archaic_mana_value_is_six` cites CR 202.3f inside the body.
#  4. **The helper pass had bug 1 all over again**, one function down: a
#     helper's body ended on the line after its signature unless the opening
#     brace was on that signature. Every helper with a multi-line parameter
#     list — `assert_fetchland_fetches`, `cast_and_resolve_at` — spliced its
#     *signature* into each caller, which carries no engine marker, so the
#     fourteen fetchland / dual-land tests in
#     `modern/lands_equipment_vehicles.rs` and two extra-turn tests in
#     `core_rules/xtra.rs` were offered up for deletion. Fixed by not closing
#     a helper until its opening brace has been seen.
#  5. The marker list had no deck-legality entry, so the whole *format* engine
#     read as "no GameState". `validate_deck` / `validate_full_deck` /
#     `validate_commander_deck` / `companion_restriction_met` take a
#     `Vec<CardDefinition>` and return errors — no game is built and none is
#     needed — and the nineteen tests that drive them (all of
#     `core_rules/format.rs`, two commander-deck validators in
#     `multiplayer.rs`, the sideboard and ante rules in `cr_recent67`/`72`,
#     and Advantageous Proclamation in `cns.rs`) were on the list.
#     **A pure-data test is one with no LOGIC under it, not one with no
#     `GameState` in it** — that is the durable form of this bug.
#
# 185 candidates at first, 284 after fixes 1-3, 250 after fix 4, **224 after
# fix 5** (209 + 15 sacred; the count also carries -8+1 for the `ogw.rs` fold).
# Fixes 4 and 5 only ever *remove* rows — no test entered the list because of
# either, which is the check to run on the next one. Of the 209, the
# single-factory definition echoes over ~60 files are the sweep the convention
# is about; the rest read several factories and are mostly the per-set
# definition tables the convention asks things to be folded *into*.
#
# **Read the file with the most hits first.** A filter's false positives
# cluster, because they share a helper or an API: fixes 4 and 5 were each
# found by reading three tests in the file at the top of the by-file count.
#
# ⚠ The awk program is in single quotes: an apostrophe in a comment inside it
# ends the shell string. It has broken this file twice. No contractions.
#
# Engine-touching markers (case-sensitive substrings).
ENGINE_MARKERS='two_player_game|multi_player_game|game_with_format|GameState::|GameAction::|add_card_to_|perform_action|do_cleanup|do_untap|drain_stack|cast\(|cast_at|battlefield|priority|check_state_based|resolve_effect|EffectContext|computed_permanent|fire_step|fire_start|register_replacement|seat_commanders|adjust_life|set_life|battlefield_find|effective_life|StackItem::|GameEvent::|granted_triggers|active_player_idx|CardInstance::new|CardInstance ::|CardInstance{|format::Format|Decklist|legality|color_identity|build_deck|cube|draft|Snapshot|view\.|view::|validate_deck|validate_full_deck|validate_commander_deck|companion_restriction_met|DeckError|Format::'

# The suite moved to its own crate when `crabomination_tests` was split out;
# this scanned `crabomination/src/tests/*.rs`, which has not existed since,
# so it silently found nothing. Recurse the real tree.
for file in $(find crabomination_tests/tests -name '*.rs'); do
  # Two passes over the same file: the first collects every non-#[test]
  # `fn name(...)` body in it, the second scans the tests and splices those
  # bodies in wherever a test names one. A test that delegates to a local
  # helper is NOT pure data, and body-only scanning cannot see that - see
  # the header.
  awk -v markers="$ENGINE_MARKERS" '
    FNR == NR {
      if (!in_h && /^#\[test\]/) { skip_next_fn = 1; next }
      if (!in_h && /^fn /) {
        if (skip_next_fn) { skip_next_fn = 0; next }
        hname = $0
        sub(/^fn /, "", hname); sub(/[(<].*$/, "", hname)
        in_h = 1; hbody = $0
        hd = 0; hopen = 0
        n = gsub(/\{/, "{", $0); hd += n; hopen += n
        n = gsub(/\}/, "}", $0); hd -= n
        # A helper is not finished until its opening brace has been SEEN.
        # Without `hopen` a multi-line signature - `fn assert_fetchland_fetches(`
        # with its two `fn() -> CardDefinition` parameters on their own lines -
        # ends the helper on line two, so the body spliced into every caller is
        # the signature and carries no engine marker. That put fourteen live
        # engine tests in `modern/lands_equipment_vehicles.rs` on a DELETE list.
        if (hopen > 0 && hd <= 0) { helper[hname] = hbody; in_h = 0; hbody = "" }
        next
      }
      if (in_h) {
        hbody = hbody "\n" $0
        n = gsub(/\{/, "{", $0); hd += n; hopen += n
        n = gsub(/\}/, "}", $0); hd -= n
        if (hopen > 0 && hd <= 0 && NF > 0) { helper[hname] = hbody; in_h = 0; hbody = "" }
      }
      next
    }
    BEGIN { in_test=0; body=""; name=""; start_line=0; doc="" }
    # Keep the run of doc/line comments immediately above a test, so the
    # sacred ones can be marked rather than silently offered up.
    !in_test && /^[[:space:]]*\/\// { doc = doc "\n" $0; next }
    !in_test && !pending && !/^#\[/ && NF > 0 { doc = "" }
    /^#\[test\]/ { pending=1; next }
    pending && /^fn / {
      name=$0
      # FNR, not NR: the file is read twice (helper pass, then scan pass), so
      # NR is the running total across both and every reported line number
      # would be off by the file length.
      start_line=FNR
      in_test=1
      body=$0
      # The signature line carries the opening brace, and counting it here
      # is what makes brace_depth mean what the block below assumes.
      # Without it the depth was 0 on the first body line, so a test ENDED
      # after one line - and every test whose first line read a catalog::
      # definition was reported as pure-data however much engine it touched
      # underneath. See the header for the before/after counts.
      brace_depth = 0
      n = gsub(/\{/, "{", $0); brace_depth += n
      n = gsub(/\}/, "}", $0); brace_depth -= n
      pending=0
      next
    }
    in_test {
      body = body "\n" $0
      n = gsub(/\{/, "{", $0); brace_depth += n
      n = gsub(/\}/, "}", $0); brace_depth -= n
      if (brace_depth <= 0 && NF > 0) {
        # End of fn.
        if (body ~ "catalog::") {
          # The test as written, before any helper is spliced in - the
          # sacred check reads this, not the expansion, so a rule citation
          # inside a shared helper cannot mark every caller sacred.
          raw = body
          # Splice in any same-file helper the test calls, twice, so a helper
          # that calls a helper is covered too. `assert_school_land` in
          # sos/hybrid_lands.rs builds a game and activates two mana
          # abilities; without this the six lands that call it read as
          # pure-data three-liners.
          for (round = 0; round < 2; round++) {
            for (h in helper) {
              if (index(body, h "(") > 0 && index(body, helper[h]) == 0) {
                body = body "\n" helper[h]
              }
            }
          }
          # Check no engine marker is present.
          # awk does not do alternation in gsub easily; emulate with split.
          n = split(markers, pat, /\|/)
          touched = 0
          for (i=1; i<=n; i++) {
            if (index(body, pat[i]) > 0) { touched=1; break }
          }
          if (!touched) {
            sub(/^fn /, "", name)
            sub(/\(.*$/, "", name)
            # A citation inside the body counts as much as one above it:
            # magmablood_archaic_mana_value_is_six pins CR 202.3f in a
            # comment on the assert, and a doc-comment-only check offered it
            # up for deletion.
            sacred = (doc ~ /CR [0-9]/ || raw ~ /CR [0-9]/ \
                      || tolower(doc) ~ /ruling|bug fix|regression/ \
                      || tolower(raw) ~ /ruling|bug fix|regression/)
            print (sacred ? "[CR] " : "     ") FILENAME ":" start_line " " name
          }
          doc = ""
        }
        in_test=0
        body=""
      }
    }
  ' "$file" "$file"
done
