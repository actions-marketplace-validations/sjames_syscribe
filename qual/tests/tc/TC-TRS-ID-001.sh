tc_TRS_ID_001() {
    local F="$1"
    run_scenario "valid REQ-* id patterns are accepted" "$F/TC-TRS-ID-001/valid-req-ids"
    assert_no_code "E006"
    assert_exit_zero

    run_scenario "invalid REQ-* id pattern produces E006" "$F/TC-TRS-ID-001/invalid-req-ids"
    assert_has_code "E006"
    assert_exit_nonzero
}
