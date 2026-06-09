tc_TRS_PROJ_006() {
    local F="$1"; local V="$F/TC-TRS-PROJ-006/variant"

    # --- Scenario: each analysis command accepts --config and rejects a bad one ---
    SCENARIO_NAME="analysis commands honour --config (valid 0, bad non-zero)"
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    local cmd
    for cmd in metrics cyber-risk co-analysis verification-depth safety-case; do
        "$SYSCRIBE" -m "$V" "$cmd" --config CONF-BASE-001 >/dev/null 2>&1 && ok=0 || ok=$?
        [ "${ok:-0}" -eq 0 ] && pass "$cmd --config CONF-BASE-001 exits 0" \
            || fail "$cmd --config exit ${ok} (expected 0)"
        "$SYSCRIBE" -m "$V" "$cmd" --config NOPE-X >/dev/null 2>&1 && bad=0 || bad=$?
        [ "${bad:-0}" -ne 0 ] && pass "$cmd --config NOPE-X exits non-zero" \
            || fail "$cmd bad --config did not error"
    done

    # --- Scenario: verification-depth is computed over the projected subset ---
    SCENARIO_NAME="verification-depth projects out a gated SIL-3 requirement"
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$V" verification-depth 2>/dev/null | grep -qF "REQ-ADAS-001" \
        && pass "whole-model verification-depth lists the gated requirement" \
        || fail "REQ-ADAS-001 missing from whole-model verification-depth"
    "$SYSCRIBE" -m "$V" verification-depth --config CONF-BASE-001 2>/dev/null | grep -qF "REQ-ADAS-001" \
        && fail "REQ-ADAS-001 still present under --config (not projected out)" \
        || pass "verification-depth --config omits the projected-out requirement"
}
