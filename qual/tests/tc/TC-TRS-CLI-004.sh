tc_TRS_CLI_004() {
    local F="$1"; local B="$F/TC-TRS-CLI-004"

    # Auto-discovery: from a subdirectory, no -m / no env, walk up to .syscribe.toml.
    SCENARIO_NAME="a command run from a subdirectory discovers the marked root"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local ec
    SCENARIO_OUTPUT=$(cd "$B/root/sub" && unset SYSCRIBE_MODEL; "$SYSCRIBE" list PartDef 2>/dev/null) && ec=0 || ec=$?
    printf '%s' "$SCENARIO_OUTPUT" | grep -qF "Engine" && pass "discovered root lists Engine" || fail "did not discover marked root"
    [ "${ec:-0}" -eq 0 ] && pass "exit 0 on discovery" || fail "exit ${ec} (expected 0)"

    # Explicit -m overrides discovery even from inside a marked model.
    SCENARIO_NAME="an explicit -m overrides discovery"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$(cd "$B/root/sub" && unset SYSCRIBE_MODEL; "$SYSCRIBE" -m "$B/other" list PartDef 2>/dev/null || true)
    if printf '%s' "$SCENARIO_OUTPUT" | grep -qF "Widget" && ! printf '%s' "$SCENARIO_OUTPUT" | grep -qF "Engine"; then
        pass "-m overrides discovery (Widget, not Engine)"
    else
        fail "explicit -m did not override discovery"
    fi

    # Back-compat: explicit -m works on a model that has NO .syscribe.toml, run
    # from a directory that itself has no marker (the existing default behaviour).
    SCENARIO_NAME="explicit -m works when the model has no .syscribe.toml"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$(cd "$B/nomarker/sub" && unset SYSCRIBE_MODEL; "$SYSCRIBE" -m "$B/other" list PartDef 2>/dev/null) && ec=0 || ec=$?
    printf '%s' "$SCENARIO_OUTPUT" | grep -qF "Widget" && pass "explicit -m lists the model with no marker" || fail "explicit -m failed without a marker"
    [ "${ec:-0}" -eq 0 ] && pass "exit 0 with explicit -m and no marker" || fail "exit ${ec} (expected 0)"

    # No marker in any ancestor and no model/ -> fall back to the default, report the miss.
    SCENARIO_NAME="no marker and no model falls back to default and reports the miss"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$(cd "$B/nomarker/sub" && unset SYSCRIBE_MODEL; "$SYSCRIBE" validate 2>&1) && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "exits non-zero on fallback miss" || fail "did not fall back (exit ${ec})"
}
