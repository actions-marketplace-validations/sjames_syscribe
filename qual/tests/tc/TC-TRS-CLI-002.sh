tc_TRS_CLI_002() {
    # Non-existent path — no fixture needed; capture exit code without || true
    _flush_scenario
    SCENARIO_NAME="non-existent path exits non-zero"
    _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m /nonexistent/path/for/qual/test 2>&1) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_exit_nonzero
}
