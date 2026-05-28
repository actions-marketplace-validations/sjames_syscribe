tc_TRS_PARSE_009() {
    local F="$1"
    run_scenario "file with no type: field is skipped with a warning" "$F/TC-TRS-PARSE-009/missing-type"
    # W008 or similar — not counted as model element, no fatal error
    assert_exit_zero

    run_scenario "file with type: present is processed normally" "$F/valid-model"
    assert_no_code "E005"
    assert_exit_zero
}
