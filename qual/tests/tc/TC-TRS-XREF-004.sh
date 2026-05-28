tc_TRS_XREF_004() {
    local F="$1"
    run_scenario "two-element supertype cycle is detected without crashing" "$F/TC-TRS-XREF-004/two-cycle"
    # Tool must exit normally; should report some error
    assert_exit_nonzero
}
