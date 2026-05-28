tc_TRS_PARSE_007() {
    local F="$1"
    run_scenario "file with --- on first line is parsed correctly" "$F/valid-model"
    assert_no_code "E001"
    assert_exit_zero

    run_scenario "file with blank first line produces E001 (missing frontmatter)" "$F/TC-TRS-PARSE-007/blank-first-line"
    assert_has_code "E001"
    assert_exit_nonzero
}
