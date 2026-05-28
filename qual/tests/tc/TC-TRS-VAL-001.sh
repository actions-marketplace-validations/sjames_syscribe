tc_TRS_VAL_001() {
    local F="$1"
    # Each parse-time error code gets its own scenario
    local -a codes=(E001 E002 E004 E005 E006 E007 E008 E009 E010 E011 E012 E013 E014 E015 E300 E301 E302 E303 E304)
    for code in "${codes[@]}"; do
        run_scenario "trigger $code" "$F/TC-TRS-VAL-001/$code"
        assert_has_code "$code"
    done
}
