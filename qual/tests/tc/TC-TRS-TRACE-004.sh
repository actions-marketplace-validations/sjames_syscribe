tc_TRS_TRACE_004() {
    local F="$1"
    run_scenario "approved leaf requirement with no satisfies produces W300" "$F/TC-TRS-TRACE-004/unassigned-approved-leaf"
    assert_has_code "W300"

    run_scenario "approved leaf requirement with satisfies produces no W300" "$F/TC-TRS-TRACE-004/assigned-approved-leaf"
    assert_no_code "W300"
    assert_exit_zero
}
