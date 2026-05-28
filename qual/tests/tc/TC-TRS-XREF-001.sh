tc_TRS_XREF_001() {
    local F="$1"
    run_scenario "absolute supertype reference resolves correctly" "$F/TC-TRS-XREF-001/abs-ref-resolved"
    assert_no_code "E103"
    assert_exit_zero

    run_scenario "absolute reference to non-existent element produces an error" "$F/TC-TRS-XREF-001/abs-ref-unresolved"
    assert_exit_nonzero
}
