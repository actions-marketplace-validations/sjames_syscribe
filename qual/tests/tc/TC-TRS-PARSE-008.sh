tc_TRS_PARSE_008() {
    local F="$1"
    run_scenario "valid YAML frontmatter produces no E002" "$F/TC-TRS-PARSE-008/valid-yaml"
    assert_no_code "E002"
    assert_exit_zero

    run_scenario "invalid YAML frontmatter produces E002" "$F/TC-TRS-PARSE-008/invalid-yaml"
    assert_has_code "E002"
    assert_exit_nonzero
}
