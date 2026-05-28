tc_TRS_PARSE_001() {
    local F="$1"
    run_scenario "valid directory is accepted" "$F/valid-model"
    assert_stdout_nonempty
    assert_exit_zero

    run_scenario "empty directory produces zero elements" "$F/TC-TRS-PARSE-001/empty"
    assert_output_contains "Total elements | 0"
    assert_exit_zero
}
