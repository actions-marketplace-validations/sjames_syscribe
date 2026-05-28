tc_TRS_ID_002() {
    local F="$1"
    run_scenario "valid TC-* id pattern is accepted" "$F/TC-TRS-ID-002/valid-tc-ids"
    assert_no_code "E006"
    assert_exit_zero

    run_scenario "invalid TC-* id pattern produces E006" "$F/TC-TRS-ID-002/invalid-tc-ids"
    assert_has_code "E006"
    assert_exit_nonzero
}
