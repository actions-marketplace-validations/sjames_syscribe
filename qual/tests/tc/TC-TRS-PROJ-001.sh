tc_TRS_PROJ_001() {
    local F="$1"; local B="$F/TC-TRS-PROJ-001/lens"
    local NOFM="$F/TC-TRS-FM-001/no-fm"

    SCENARIO_NAME="lens filters to active elements (stored config)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local w; w=$("$SYSCRIBE" -m "$B" list Requirement --config CONF-PROJ1-WDT-001 2>/dev/null || true)
    printf '%s' "$w" | grep -qF "REQ-PROJ1-CORE-001" && pass "core req present in Wdt variant" || fail "core req missing"
    printf '%s' "$w" | grep -qF "REQ-PROJ1-WDT-001" && pass "Wdt req present in Wdt variant" || fail "Wdt req missing in Wdt variant"
    local n; n=$("$SYSCRIBE" -m "$B" list Requirement --config CONF-PROJ1-NOWDT-001 2>/dev/null || true)
    printf '%s' "$n" | grep -qF "REQ-PROJ1-WDT-001" && fail "Wdt req wrongly present in no-Wdt variant" || pass "Wdt req excluded in no-Wdt variant"

    SCENARIO_NAME="ad-hoc feature selection"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local a; a=$("$SYSCRIBE" -m "$B" list Requirement --config "Features::Wdt" 2>/dev/null || true)
    printf '%s' "$a" | grep -qF "REQ-PROJ1-WDT-001" && pass "ad-hoc selection activates Wdt req" || fail "ad-hoc selection did not work"

    SCENARIO_NAME="dormant with no feature model"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local plain conf
    plain=$("$SYSCRIBE" -m "$NOFM" list Requirement 2>/dev/null || true)
    conf=$("$SYSCRIBE" -m "$NOFM" list Requirement --config CONF-X 2>/dev/null || true)
    [ "$plain" = "$conf" ] && pass "--config is inert with no feature model" || fail "--config changed output on a no-feature model"

    SCENARIO_NAME="unresolved configuration errors"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$B" list Requirement --config CONF-DOES-NOT-EXIST >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "unresolved --config exits non-zero" || fail "unresolved --config did not error"
}
