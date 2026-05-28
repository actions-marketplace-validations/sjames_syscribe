tc_TRS_ELEM_003() {
    local F="$1"
    run_scenario "PartDef with no supertype: loads without E004" "$F/valid-model"
    assert_no_code "E004"
    assert_exit_zero
}
