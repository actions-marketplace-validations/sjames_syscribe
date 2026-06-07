tc_TRS_PROJ_005() {
    local F="$1"; local B="$F/TC-TRS-PROJ-005/family"

    SCENARIO_NAME="validate --all-configs gates on any variant error"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local out; out=$("$SYSCRIBE" -m "$B" validate --all-configs 2>/dev/null || true)
    printf '%s' "$out" | grep -qF "CONF-PROJ5-A-001" && printf '%s' "$out" | grep -qF "CONF-PROJ5-B-001" \
        && pass "both configurations listed" || fail "all-configs did not list both variants"
    "$SYSCRIBE" -m "$B" validate --all-configs >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "all-configs exits non-zero (variant A has a structural escape)" || fail "all-configs did not gate"

    SCENARIO_NAME="dead element (W021)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B" feature-check --deep 2>/dev/null || true)
    assert_has_code "W021"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W021 |" | grep -qF "GhostPart" \
        && pass "W021 names the dead element" || fail "W021 does not name GhostPart"

    SCENARIO_NAME="aggregate coverage gap (W022)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_has_code "W022"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W022 |" | grep -qF "REQ-PROJ5-WDT-001" \
        && pass "W022 names the never-covered requirement" || fail "W022 does not name REQ-PROJ5-WDT-001"

    SCENARIO_NAME="variant diff lists the symmetric difference"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local d; d=$("$SYSCRIBE" -m "$B" diff --config CONF-PROJ5-A-001 --config CONF-PROJ5-B-001 2>/dev/null || true)
    printf '%s' "$d" | grep -qF "REQ-PROJ5-WDT-001" && printf '%s' "$d" | grep -qF "REQ-PROJ5-MPS-001" \
        && pass "diff lists requirements unique to each variant" || fail "diff did not list the symmetric difference"
}
