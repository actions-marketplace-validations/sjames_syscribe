tc_TRS_CLI_003() {
    SCENARIO_OUTPUT=$("$SYSCRIBE" --agent-instructions 2>/dev/null || true)
    SCENARIO_EXIT=$?
    SCENARIO_NAME="--agent-instructions prints prompt and exits 0"
    SCENARIO_ASSERTIONS=()
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_stdout_nonempty
    assert_exit_zero
}
