tc_TRS_ELEM_001() {
    local F="$1"
    run_scenario "all defined element types are recognised without E005" "$F/TC-TRS-ELEM-001/all-types"
    assert_no_code "E005"
    assert_exit_zero
}
