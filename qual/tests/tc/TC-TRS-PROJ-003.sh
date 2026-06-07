tc_TRS_PROJ_003() {
    local F="$1"; local B="$F/TC-TRS-PROJ-003/escape"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B" validate --config CONF-PROJ3-001 2>/dev/null || true)

    SCENARIO_NAME="structural escape is an error (E226), not a dangling-ref error"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_has_code "E226"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E226 |" | grep -qF "M0Module" \
        && pass "E226 names the inactive structural target" || fail "E226 does not name M0Module"
    assert_no_code "E102"

    SCENARIO_NAME="traceability escape is a warning (W019)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_has_code "W019"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W019 |" | grep -qF "REQ-PROJ3-WDT-001" \
        && pass "W019 names the inactive verified requirement" || fail "W019 does not name the requirement"

    SCENARIO_NAME="references between active elements do not escape"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -E "E226|W019" | grep -qF "CoreCtrl" \
        && fail "active→active reference wrongly flagged" || pass "active→active reference not flagged"
}
