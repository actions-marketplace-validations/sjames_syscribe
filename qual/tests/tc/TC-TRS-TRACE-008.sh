tc_TRS_TRACE_008() {
    local F="$1"
    run_scenario "deployment package with no hardware allocation produces E314" "$F/TC-TRS-TRACE-008/no-hw-allocation"
    assert_has_code "E314"
    assert_exit_nonzero

    run_scenario "deployment package with hardware allocation produces no E314" "$F/TC-TRS-TRACE-008/with-hw-allocation"
    assert_no_code "E314"
    assert_exit_zero
}
