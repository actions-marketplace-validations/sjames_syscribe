tc_TRS_VAL_006() {
    local F="$1"
    run_scenario "E-code findings are under Errors section" "$F/TC-TRS-VAL-001/E005"
    assert_has_code "E005"
    assert_output_contains "### Errors"
    assert_exit_nonzero

    run_scenario "W-code findings are under Warnings section" "$F/TC-TRS-VAL-003/W005"
    assert_has_code "W005"
    assert_output_contains "### Warnings"
}
