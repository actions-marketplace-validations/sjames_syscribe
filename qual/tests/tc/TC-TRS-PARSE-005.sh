tc_TRS_PARSE_005() {
    local F="$1"
    run_scenario "_index.md is treated as package declaration" "$F/TC-TRS-PARSE-005/index-as-package"
    # Pkg/Foo.md should load as VS::Foo (name: VS in _index.md)
    assert_no_code "E005"
    assert_exit_zero
}
