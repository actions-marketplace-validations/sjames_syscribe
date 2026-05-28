tc_TRS_ELEM_002() {
    local F="$1"
    run_scenario "completely unknown type value produces E005" "$F/TC-TRS-ELEM-002/unknown-type"
    assert_has_code "E005"
    assert_exit_nonzero

    run_scenario "wrong-case type value produces E005" "$F/TC-TRS-ELEM-002/wrong-case"
    assert_has_code "E005"
    assert_exit_nonzero
}
