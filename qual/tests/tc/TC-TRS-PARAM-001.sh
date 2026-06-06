tc_TRS_PARAM_001() {
    local F="$1"; local B="$F/TC-TRS-PARAM-001"

    # All six binding rules fire in the violations model (one config per code).
    run_scenario "parameter binding violations emit E203–E206, E222, W017" "$B/violations"
    assert_has_code "E203"
    assert_has_code "E204"
    assert_has_code "E205"
    assert_has_code "E206"
    assert_has_code "E222"
    assert_has_code "W017"

    # Fully valid bindings emit none of the parameter findings.
    run_scenario "valid bindings emit no parameter findings" "$B/valid"
    assert_no_code "E203"
    assert_no_code "E204"
    assert_no_code "E205"
    assert_no_code "E206"
    assert_no_code "E222"
    assert_no_code "W017"
}
