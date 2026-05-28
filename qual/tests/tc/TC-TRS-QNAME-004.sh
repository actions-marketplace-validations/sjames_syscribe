tc_TRS_QNAME_004() {
    local F="$1"
    run_scenario "_index.md contributes no _index name segment" "$F/TC-TRS-QNAME-004/no-index-segment"
    assert_no_code "E005"
    assert_exit_zero
}
