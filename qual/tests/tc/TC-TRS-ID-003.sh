tc_TRS_ID_003() {
    local F="$1"
    run_scenario "valid ADR-* id is accepted" "$F/TC-TRS-ID-003/valid-adr"
    assert_no_code "E300"
    assert_no_code "E301"
    assert_exit_zero

    run_scenario "ADR id not matching pattern produces E300" "$F/TC-TRS-ID-003/adr-bad-pattern"
    assert_has_code "E300"
    assert_exit_nonzero

    run_scenario "ADR missing id produces E301" "$F/TC-TRS-ID-003/adr-missing-id"
    assert_has_code "E301"
    assert_exit_nonzero
}
