tc_TRS_PROJ_002() {
    local F="$1"; local B="$F/TC-TRS-PROJ-002/revalidate"

    # REQ-MPS is active in CONF-PROJ2-MPS2-001 (Mps2) but its only test is active
    # only under Wdt → uncovered in this variant, though covered in the 150% model.
    SCENARIO_NAME="requirement covered only by an inactive test is uncovered in the variant"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B" validate --config CONF-PROJ2-MPS2-001 2>/dev/null || true)
    if printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W002 |" | grep -qF "REQ-PROJ2-MPS-001"; then
        pass "lens reports W002 for REQ-PROJ2-MPS-001"
    else
        fail "lens did not report the variant coverage gap"
    fi

    SCENARIO_NAME="whole-model validate reports no such coverage gap"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    run_scenario "whole-model validate" "$B"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W002 |" | grep -qF "REQ-PROJ2-MPS-001" \
        && fail "whole-model wrongly reports the coverage gap" || pass "whole-model does not report the gap (covered in 150%)"
}
