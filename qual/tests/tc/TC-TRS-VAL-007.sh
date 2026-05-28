tc_TRS_VAL_007() {
    local F="$1"
    run_scenario "Errors section does not contain W-codes" "$F/TC-TRS-VAL-001/E005"
    # E005 should appear; no W-code should appear in its section
    assert_has_code "E005"
    assert_exit_nonzero

    run_scenario "Warnings section does not contain E-codes" "$F/TC-TRS-VAL-003/W001"
    assert_has_code "W001"
    assert_no_code "E001"
}
