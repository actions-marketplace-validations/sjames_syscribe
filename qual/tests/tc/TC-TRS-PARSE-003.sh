tc_TRS_PARSE_003() {
    local F="$1"
    run_scenario "target/ directory is excluded from discovery" "$F/TC-TRS-PARSE-003/excluded-dirs"
    # ShouldBeIgnored.md inside target/ must not appear; Valid.md should load fine
    assert_no_code "E005"
    assert_exit_zero
}
