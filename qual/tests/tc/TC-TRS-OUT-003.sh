tc_TRS_OUT_003() {
    local F="$1"
    run_scenario "summary section includes error and warning counts" "$F/TC-TRS-VAL-001/E005"
    assert_output_contains "Errors"
    assert_output_contains "Warnings"
    assert_exit_nonzero
}
