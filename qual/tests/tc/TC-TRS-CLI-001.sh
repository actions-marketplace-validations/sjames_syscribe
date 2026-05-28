tc_TRS_CLI_001() {
    local F="$1"
    run_scenario "-m flag accepts the model directory" "$F/valid-model"
    assert_stdout_nonempty
    assert_exit_zero

    # --model long form
    SCENARIO_OUTPUT=$("$SYSCRIBE" --model "$F/valid-model" 2>/dev/null)
    SCENARIO_EXIT=$?
    SCENARIO_NAME="--model long form"
    SCENARIO_ASSERTIONS=()
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_stdout_nonempty
    assert_exit_zero
}
