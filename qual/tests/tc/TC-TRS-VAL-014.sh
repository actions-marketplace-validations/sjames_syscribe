tc_TRS_VAL_014() {
    local F="$1"; local SRC="$F/TC-TRS-VAL-014/model"

    run_scenario "drift checks scoped to active TestCases" "$SRC"

    # code, filename-substring present together on one finding line?
    _line() { printf '%s' "$SCENARIO_OUTPUT" | grep -F "| $1 |" | grep -qF "$2"; }

    # Draft TestCase drift is suppressed.
    _line W004 "tc_draft_missing.md" && fail "draft missing sourceFile wrongly W004" || pass "draft missing sourceFile suppressed"
    _line W009 "tc_draft_drift.md"   && fail "draft drift wrongly W009"             || pass "draft drift suppressed"

    # Active TestCase drift is reported.
    _line W004 "tc_active_missing.md" && pass "active missing sourceFile -> W004" || fail "active missing sourceFile not flagged"
    _line W009 "tc_active_drift.md"   && pass "active drift -> W009"              || fail "active drift not flagged"

    # Non-TestCase sourceFile is always checked.
    _line W004 "Part.md" && pass "PartDef missing sourceFile -> W004" || fail "PartDef sourceFile not flagged"
}
