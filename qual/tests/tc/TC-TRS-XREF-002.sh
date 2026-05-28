tc_TRS_XREF_002() {
    local F="$1"
    run_scenario "sibling reference resolves within the same package" "$F/TC-TRS-XREF-002/rel-ref-resolved"
    assert_exit_zero
}
