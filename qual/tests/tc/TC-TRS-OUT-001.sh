tc_TRS_OUT_001() {
    local F="$1"
    run_scenario "report is written to stdout in Markdown format" "$F/valid-model"
    assert_output_contains "# "
    assert_stdout_nonempty
    assert_exit_zero
}
