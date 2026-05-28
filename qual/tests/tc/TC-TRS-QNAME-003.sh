tc_TRS_QNAME_003() {
    local F="$1"
    run_scenario "name: in frontmatter replaces the filename stem" "$F/TC-TRS-QNAME-003/element-name-override"
    assert_no_code "E005"
    assert_exit_zero
}
