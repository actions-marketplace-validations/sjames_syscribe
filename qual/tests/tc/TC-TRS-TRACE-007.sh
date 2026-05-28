tc_TRS_TRACE_007() {
    local F="$1"
    run_scenario "software PartDef with hardware supertype produces E315" "$F/TC-TRS-TRACE-007/hw-sw-supertype"
    assert_has_code "E315"
    assert_exit_nonzero
}
