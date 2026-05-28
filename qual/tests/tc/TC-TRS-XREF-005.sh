tc_TRS_XREF_005() {
    local F="$1"
    run_scenario "verifies: resolves by stable id: regardless of file path" "$F/TC-TRS-XREF-005/id-based"
    assert_no_code "E102"
    assert_exit_zero
}
