tc_TRS_TRACE_005() {
    local F="$1"
    run_scenario "architecture element satisfying a parent requirement produces E312" "$F/TC-TRS-TRACE-005/parent-in-satisfies"
    assert_has_code "E312"
    assert_exit_nonzero
}
