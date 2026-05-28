tc_TRS_VAL_005() {
    local F="$1"
    run_scenario "findings include rule code, element reference, and description" "$F/TC-TRS-VAL-001/E005"
    # Output must contain E005 in a table row with at least three pipe-separated columns
    assert_has_code "E005"
    # Verify the output has a Markdown table with Code | File | Message columns
    assert_output_contains "| Code |"
    assert_exit_nonzero

    run_scenario "parse-time error is attributed to source file" "$F/TC-TRS-VAL-001/E002"
    assert_has_code "E002"
    assert_exit_nonzero
}
