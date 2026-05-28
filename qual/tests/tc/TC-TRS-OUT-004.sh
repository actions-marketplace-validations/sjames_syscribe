tc_TRS_OUT_004() {
    local F="$1"
    run_scenario "model with errors exits non-zero" "$F/TC-TRS-VAL-001/E005"
    assert_has_code "E005"
    assert_exit_nonzero

    run_scenario "model with errors and warnings still exits non-zero" "$F/TC-TRS-VAL-001/E006"
    assert_exit_nonzero
}
