tc_TRS_TRACE_001() {
    local F="$1"
    run_scenario "verifiedBy is computed from TestCase.verifies:" "$F/TC-TRS-TRACE-001/reverse-indices"
    assert_no_code "E102"
    assert_exit_zero
}
