tc_TRS_FMA_011() {
    local F="$1"; local V="$F/TC-TRS-FMA-005/void"

    # Scenario: --prove emits a re-checkable DIMACS CNF for a void model.
    # (batsat 0.6.0 exposes no solver-recorded DRAT refutation, so the DRAT
    # *proof* is deferred; the emitted CNF is externally re-checkable as UNSAT.)
    SCENARIO_NAME="--prove emits a DIMACS CNF for a void model"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local dir; dir=$(mktemp -d)
    "$SYSCRIBE" -m "$V" feature-check --deep --prove "$dir" >/dev/null 2>&1 || true
    local ncnf; ncnf=$(find "$dir" -name '*.cnf' -size +0c 2>/dev/null | wc -l)
    [ "$ncnf" -ge 1 ] && pass "a non-empty DIMACS CNF was written" || fail "no DIMACS CNF written"
    # The emitted CNF must be a well-formed DIMACS problem.
    local f; f=$(find "$dir" -name 'void.cnf' 2>/dev/null | head -1)
    [ -n "$f" ] && head -1 "$f" | grep -qE "^p cnf [0-9]+ [0-9]+$" \
        && pass "void.cnf has a valid DIMACS header" || fail "void.cnf missing/invalid header"
    rm -rf "$dir"

    # Scenario: no proof files without --prove.
    SCENARIO_NAME="no proof files without --prove"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local dir2; dir2=$(mktemp -d)
    ( cd "$dir2" && "$SYSCRIBE" -m "$V" feature-check --deep >/dev/null 2>&1 || true )
    local n; n=$(find "$dir2" -type f 2>/dev/null | wc -l)
    [ "$n" -eq 0 ] && pass "no proof files emitted without --prove" || fail "unexpected files written without --prove (${n})"
    rm -rf "$dir2"
}
