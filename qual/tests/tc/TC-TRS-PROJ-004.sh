tc_TRS_PROJ_004() {
    local F="$1"; local B="$F/TC-TRS-PROJ-004/global"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B" feature-check --deep 2>/dev/null || true)

    SCENARIO_NAME="violable structural edge proven over all variants (E227)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_has_code "E227"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E227 |" | grep -qF "FeatModule" \
        && pass "E227 names the conditional structural target" || fail "E227 does not name FeatModule"

    SCENARIO_NAME="holding implication not flagged"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E227 |" | grep -qF "FeatModule2" \
        && fail "non-violable edge wrongly flagged (FeatModule2)" || pass "non-violable edge not flagged"

    SCENARIO_NAME="traceability edge is a warning (W020)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_has_code "W020"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W020 |" | grep -qF "REQ-PROJ4-FEAT-001" \
        && pass "W020 names the conditional verified requirement" || fail "W020 does not name the requirement"
}
