tc_TRS_TRACE_002() {
    local F="$1"
    run_scenario "derivedFrom with no breakdownAdr produces E310" "$F/TC-TRS-TRACE-002/no-breakdown-adr"
    assert_has_code "E310"
    assert_exit_nonzero

    run_scenario "derivedFrom with valid breakdownAdr produces no E310" "$F/TC-TRS-TRACE-002/with-breakdown-adr"
    assert_no_code "E310"
    assert_exit_zero
}
