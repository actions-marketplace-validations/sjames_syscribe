tc_TRS_OUT_002() {
    local F="$1"
    run_scenario "findings table has Code, File, and Message columns" "$F/TC-TRS-VAL-001/E005"
    assert_has_code "E005"
    assert_output_contains "| Code |"
    assert_output_contains "| Message |"
    assert_exit_nonzero
}
