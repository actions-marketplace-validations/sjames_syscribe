tc_TRS_PARSE_006() {
    local F="$1"
    run_scenario "malformed YAML frontmatter produces a warning, not a fatal error" "$F/TC-TRS-PARSE-006/malformed-plus-valid"
    # BadYAML.md triggers E002; Valid.md still counted
    assert_has_code "E002"
    assert_exit_nonzero
}
