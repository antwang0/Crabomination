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
# Engine-touching markers (case-sensitive substrings).
ENGINE_MARKERS='two_player_game|multi_player_game|game_with_format|GameState::|GameAction::|add_card_to_|perform_action|do_cleanup|do_untap|drain_stack|cast\(|cast_at|battlefield|priority|check_state_based|resolve_effect|EffectContext|computed_permanent|fire_step|fire_start|register_replacement|seat_commanders|adjust_life|set_life|battlefield_find|effective_life|StackItem::|GameEvent::|granted_triggers|active_player_idx|CardInstance::new|CardInstance ::|CardInstance{|format::Format|Decklist|legality|color_identity|build_deck|cube|draft|Snapshot|view\.|view::'

# The suite moved to its own crate when `crabomination_tests` was split out;
# this scanned `crabomination/src/tests/*.rs`, which has not existed since,
# so it silently found nothing. Recurse the real tree.
for file in $(find crabomination_tests/tests -name '*.rs'); do
  awk -v markers="$ENGINE_MARKERS" '
    BEGIN { in_test=0; body=""; name=""; start_line=0; doc="" }
    # Keep the run of doc/line comments immediately above a test, so the
    # sacred ones can be marked rather than silently offered up.
    !in_test && /^[[:space:]]*\/\// { doc = doc "\n" $0; next }
    !in_test && !pending && !/^#\[/ && NF > 0 { doc = "" }
    /^#\[test\]/ { pending=1; next }
    pending && /^fn / {
      name=$0
      start_line=NR
      in_test=1
      body=""
      brace_depth=0
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
            sacred = (doc ~ /CR [0-9]/ || tolower(doc) ~ /ruling|bug fix|regression/)
            print (sacred ? "[CR] " : "     ") FILENAME ":" start_line " " name
          }
          doc = ""
        }
        in_test=0
        body=""
      }
    }
  ' "$file"
done
