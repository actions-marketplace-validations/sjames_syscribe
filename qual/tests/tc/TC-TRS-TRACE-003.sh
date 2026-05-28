tc_TRS_TRACE_003() {
    local F="$1"
    run_scenario "approved requirement with proposed ADR produces W303" "$F/TC-TRS-TRACE-003/proposed-adr-approved-req"
    assert_has_code "W303"
}
