tc_TRS_TRACE_006() {
    local F="$1"
    run_scenario "software element satisfying hardware requirement produces E313" "$F/TC-TRS-TRACE-006/incompatible-domain"
    assert_has_code "E313"
    assert_exit_nonzero

    run_scenario "software element satisfying system requirement produces no E313" "$F/TC-TRS-TRACE-006/compatible-domain"
    assert_no_code "E313"
    assert_exit_zero
}
