tc_TRS_PARAM_004() {
    local F="$1"; local B="$F/TC-TRS-PARAM-004"

    # E230: an unknown bindingTime value is rejected by validate.
    run_scenario "an unknown bindingTime value produces E230" "$B/e230"
    assert_has_code "E230"

    # E229: a compile-time parameter derivedFrom a runtime sibling cannot be bound
    # earlier than the value it depends on (holistic — feature-check).
    SCENARIO_NAME="a parameter bound earlier than its derivedFrom source produces E229"
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/ordering" feature-check 2>/dev/null || true)
    assert_has_code "E229"

    SCENARIO_NAME="a parameter bound no earlier than its source produces no E229"
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/ordering_ok" feature-check 2>/dev/null || true)
    assert_no_code "E229"

    # W027: binding a runtime parameter in a Configuration warns.
    run_scenario "binding a runtime parameter in a Configuration produces W027" "$B/w027"
    assert_has_code "W027"

    # W017 suppression: a required runtime parameter left unbound does not warn.
    run_scenario "a required unbound runtime parameter suppresses W017" "$B/w017_runtime"
    assert_no_code "W017"

    # ... but a required non-runtime parameter left unbound still warns.
    run_scenario "a required unbound non-runtime parameter still produces W017" "$B/w017_compile"
    assert_has_code "W017"

    # Clean model: well-ordered binding times produce none of the findings.
    run_scenario "well-ordered binding times emit no binding-time findings" "$B/valid"
    assert_no_code "E230"
    assert_no_code "E229"
    assert_no_code "W027"
    assert_no_code "W017"
}
