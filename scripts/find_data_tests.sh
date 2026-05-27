#!/usr/bin/env bash
# Find pure-data tests: tests that only read CardDefinition fields without
# any GameState interaction. Outputs `file:line fn_name` for each candidate.
#
# A pure-data test:
#  - contains `let _ = catalog::*()` or `catalog::*()` calls
#  - does NOT contain any GameState-touching identifier
#
# Engine-touching markers (case-sensitive substrings).
ENGINE_MARKERS='two_player_game|multi_player_game|game_with_format|GameState::|GameAction::|add_card_to_|perform_action|do_cleanup|do_untap|drain_stack|cast\(|cast_at|battlefield|priority|check_state_based|resolve_effect|EffectContext|computed_permanent|fire_step|fire_start|register_replacement|seat_commanders|adjust_life|set_life|battlefield_find|effective_life|StackItem::|GameEvent::|granted_triggers|active_player_idx|CardInstance::new|CardInstance ::|CardInstance{|format::Format|Decklist|legality|color_identity|build_deck|cube|draft|Snapshot|view\.|view::'

for file in crabomination/src/tests/*.rs; do
  awk -v markers="$ENGINE_MARKERS" '
    BEGIN { in_test=0; body=""; name=""; start_line=0 }
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
            print FILENAME ":" start_line " " name
          }
        }
        in_test=0
        body=""
      }
    }
  ' "$file"
done
