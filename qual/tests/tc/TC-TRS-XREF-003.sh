tc_TRS_XREF_003() {
    local F="$1"
    run_scenario "dangling reference produces an error without crashing" "$F/TC-TRS-XREF-003/dangling-ref"
    # Tool must exit normally (not crash), and produce an error for the dangling ref
    assert_exit_nonzero
}
