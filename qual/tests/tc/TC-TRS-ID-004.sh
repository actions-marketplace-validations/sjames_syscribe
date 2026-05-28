tc_TRS_ID_004() {
    local F="$1"
    run_scenario "two elements with the same id produce E101" "$F/TC-TRS-ID-004/duplicate-ids"
    assert_has_code "E101"
    assert_exit_nonzero

    run_scenario "unique ids produce no E101" "$F/valid-model"
    assert_no_code "E101"
    assert_exit_zero
}
