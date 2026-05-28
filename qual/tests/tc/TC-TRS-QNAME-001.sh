tc_TRS_QNAME_001() {
    local F="$1"
    run_scenario "single-level element has single-segment qualified name" "$F/TC-TRS-QNAME-001/single-level"
    assert_no_code "E005"
    assert_exit_zero

    run_scenario "three-level nested element has three-segment qualified name" "$F/TC-TRS-QNAME-001/three-level"
    assert_no_code "E005"
    assert_exit_zero
}
