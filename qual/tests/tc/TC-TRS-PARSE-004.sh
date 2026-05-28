tc_TRS_PARSE_004() {
    local F="$1"
    run_scenario ".sysmlignore suppresses matching files" "$F/TC-TRS-PARSE-004/with-sysmlignore"
    # Draft/Excluded.md is ignored; Valid.md loads fine
    assert_exit_zero

    run_scenario "absence of .sysmlignore causes no error" "$F/valid-model"
    assert_exit_zero
}
