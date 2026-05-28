tc_TRS_PARSE_002() {
    local F="$1"
    run_scenario "elements in nested directories are discovered" "$F/TC-TRS-PARSE-002/nested"
    # A/B/Deep.md should be present → element count > 0
    assert_output_contains "Total elements"
    assert_no_code "E005"
    assert_exit_zero

    run_scenario "non-.md files are ignored" "$F/TC-TRS-PARSE-002/mixed"
    # notes.txt must not be counted as an element
    assert_no_code "E005"
    assert_exit_zero
}
