tc_TRS_OUT_005() {
    local F="$1"
    run_scenario "model with only warnings exits 0" "$F/TC-TRS-VAL-003/W005"
    assert_has_code "W005"
    assert_exit_zero

    run_scenario "clean model exits 0" "$F/valid-model"
    assert_exit_zero
}
