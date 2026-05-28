tc_TRS_QNAME_002() {
    local F="$1"
    run_scenario "name: in _index.md replaces directory name in qualified names" "$F/TC-TRS-QNAME-002/name-override"
    assert_no_code "E005"
    assert_exit_zero
}
