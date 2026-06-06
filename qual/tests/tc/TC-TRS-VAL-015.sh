tc_TRS_VAL_015() {
    local F="$1"; local SRC="$F/TC-TRS-VAL-015/model"

    run_scenario "draft emits I010 not W004; retired emits nothing" "$SRC"
    _line() { printf '%s' "$SCENARIO_OUTPUT" | grep -F "| $1 |" | grep -qF "$2"; }

    # Draft planned source -> I010, not W004.
    _line I010 "tc_draft_planned.md" && pass "draft planned -> I010" || fail "draft planned missing I010"
    _line W004 "tc_draft_planned.md" && fail "draft planned wrongly W004" || pass "draft planned not W004"

    # Retired -> nothing.
    { _line W004 "tc_retired_gone.md" || _line W009 "tc_retired_gone.md" || _line I010 "tc_retired_gone.md"; } \
        && fail "retired wrongly flagged" || pass "retired emits nothing"

    # I010 alone does not fail validation.
    SCENARIO_NAME="I010 is exit-neutral"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$SRC" validate >/dev/null 2>&1 && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    [ "$SCENARIO_EXIT" -eq 0 ] && pass "validate exits 0 with I010 present" || fail "exit $SCENARIO_EXIT (expected 0)"

    # I010 can be gated explicitly.
    SCENARIO_NAME="--deny I010 gates"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$SRC" validate --deny I010 >/dev/null 2>&1 && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    [ "$SCENARIO_EXIT" -eq 2 ] && pass "--deny I010 exits 2" || fail "exit $SCENARIO_EXIT (expected 2)"
}
