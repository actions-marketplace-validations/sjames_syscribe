tc_TRS_VAL_013() {
    local F="$1"; local SRC="$F/TC-TRS-VAL-013/model"

    # Scenario 1: hook is inert without --fetch-remote (no command execution).
    run_scenario "hook inert without --fetch-remote" "$SRC"
    assert_no_code "W004"
    assert_no_code "W009"

    # Work on a copy so the download cache never lands in the repo fixture.
    local W; W=$(mktemp -d); cp -r "$SRC"/. "$W"/
    SCENARIO_NAME="--fetch-remote fetches and verifies"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$W" validate --fetch-remote 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?

    # Drift: fetched file defines remote_present, not remote_renamed -> W009.
    printf '%s' "$SCENARIO_OUTPUT" | grep -qF "remote_renamed" \
        && pass "W009 flags missing function in fetched remote" \
        || fail "expected W009 for remote_renamed"

    # Passing function in a fetched file is not flagged.
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "W009" | grep -qF "remote_present" \
        && fail "remote_present wrongly flagged" \
        || pass "fetched passing function not flagged"

    # Retrieval failure -> W004.
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "W004" | grep -qF "could not be retrieved" \
        && pass "W004 on retrieval failure" \
        || fail "expected W004 for unretrievable remote"

    rm -rf "$W"
}
