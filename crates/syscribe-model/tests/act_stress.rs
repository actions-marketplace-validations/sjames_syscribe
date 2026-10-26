//! Stress tests for the action-flow interpreter: nested fork/join, merges that multiply tokens, loops built from
//! back-edges, accepts in parallel branches, terminate, deep call chains and loop bounds.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::act_ir::{lower_action, ActionProgram};
use syscribe_model::act_run::{ActOptions, ActRun, Status};
use syscribe_model::expr::{format_value, parse_value, Value};
use syscribe_model::walker::walk_model;

fn program(files: &[(&str, &str)], root: &str) -> ActionProgram {
    static N: AtomicU64 = AtomicU64::new(0);
    let dir: PathBuf = std::env::temp_dir().join(format!("syscribe-actstress-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (n, body) in files {
        std::fs::write(dir.join(format!("{n}.md")), format!("---\ntype: ActionDef\nname: {n}\n{body}---\n")).unwrap();
    }
    lower_action(&walk_model(&dir).unwrap(), root).unwrap_or_else(|d| panic!("lowering failed: {d:?}"))
}

fn val(s: &str) -> Value {
    parse_value(s).unwrap()
}

fn num(r: &ActRun, name: &str) -> String {
    format_value(&r.var(name).unwrap_or_else(|| panic!("{name} unset")))
}

fn run(p: &ActionProgram) -> ActRun<'_> {
    let mut r = ActRun::new(p, ActOptions::default());
    r.run();
    r
}

fn order(r: &ActRun) -> Vec<String> {
    r.trace().iter().map(|s| s.node.clone()).collect()
}

// ── fork / join ──────────────────────────────────────────────────────────────

#[test]
fn nested_fork_join_pairs_up_inner_before_outer() {
    let p = program(
        &[(
            "N",
            "variables:
  - {name: n, value: \"0\"}
subActions:
  - {name: a, kind: AssignmentAction, assign: \"n := n + 1\"}
  - {name: b, kind: AssignmentAction, assign: \"n := n + 10\"}
  - {name: c, kind: AssignmentAction, assign: \"n := n + 100\"}
  - {name: after_inner, kind: AssignmentAction, assign: \"n := n * 2\"}
  - {name: end, kind: AssignmentAction, assign: \"n := n + 1000\"}
controlNodes:
  - {name: fo, kind: ForkNode}
  - {name: fi, kind: ForkNode}
  - {name: ji, kind: JoinNode}
  - {name: jo, kind: JoinNode}
successionConnections:
  - {after: start, before: fo}
  - {after: fo, before: a}
  - {after: fo, before: fi}
  - {after: fi, before: b}
  - {after: fi, before: c}
  - {after: b, before: ji}
  - {after: c, before: ji}
  - {after: ji, before: after_inner}
  - {after: after_inner, before: jo}
  - {after: a, before: jo}
  - {after: jo, before: end}
",
        )],
        "N",
    );
    let r = run(&p);
    assert_eq!(r.status(), Status::Completed, "{:?}", r.error());
    assert!(r.ran("end"));
    let o = order(&r);
    let pos = |x: &str| o.iter().position(|s| s == x).unwrap_or_else(|| panic!("{x} missing in {o:?}"));
    assert!(pos("ji") > pos("b") && pos("ji") > pos("c"));
    assert!(pos("jo") > pos("after_inner") && pos("jo") > pos("a"));
    assert!(pos("end") > pos("jo"));
    assert_eq!(o.iter().filter(|s| *s == "end").count(), 1, "the outer join fires once");
}

#[test]
fn a_wide_fork_joins_all_fifty_branches() {
    let mut subs = String::new();
    let mut succ = String::from("  - {after: start, before: f}\n  - {after: j, before: fin}\n");
    for i in 0..50 {
        subs += &format!("  - {{name: b{i}, kind: AssignmentAction, assign: \"n := n + 1\"}}\n");
        succ += &format!("  - {{after: f, before: b{i}}}\n  - {{after: b{i}, before: j}}\n");
    }
    let body = format!(
        "variables:\n  - {{name: n, value: \"0\"}}\nsubActions:\n{subs}  - {{name: fin, kind: AssignmentAction, assign: \"n := n * 2\"}}\ncontrolNodes:\n  - {{name: f, kind: ForkNode}}\n  - {{name: j, kind: JoinNode}}\nsuccessionConnections:\n{succ}"
    );
    let p = program(&[("W", &body)], "W");
    let r = run(&p);
    assert_eq!(r.status(), Status::Completed, "{:?}", r.error());
    assert_eq!(num(&r, "n"), "100", "50 increments, then doubled once after the join");
}

#[test]
fn a_join_whose_branch_was_cut_by_a_decision_deadlocks_rather_than_completing() {
    let p = program(
        &[(
            "D",
            "variables:
  - {name: go, value: \"0\"}
subActions:
  - {name: a, kind: AssignmentAction, assign: \"go := go\"}
  - {name: b, kind: AssignmentAction, assign: \"go := go\"}
  - {name: end, kind: AssignmentAction, assign: \"go := 9\"}
controlNodes:
  - {name: f, kind: ForkNode}
  - {name: d, kind: DecisionNode}
  - {name: j, kind: JoinNode}
successionConnections:
  - {after: start, before: f}
  - {after: f, before: a}
  - {after: f, before: d}
  - {after: d, before: b, guard: \"go == 1\"}
  - {after: a, before: j}
  - {after: b, before: j}
  - {after: j, before: end}
",
        )],
        "D",
    );
    let r = run(&p);
    assert_eq!(r.status(), Status::Deadlocked);
    assert!(!r.ran("end"));
}

#[test]
fn a_merge_after_a_fork_fires_once_per_token() {
    let p = program(
        &[(
            "M",
            "variables:
  - {name: n, value: \"0\"}
subActions:
  - {name: a, kind: AssignmentAction, assign: \"n := n + 1\"}
  - {name: b, kind: AssignmentAction, assign: \"n := n + 1\"}
  - {name: tail, kind: AssignmentAction, assign: \"n := n + 10\"}
controlNodes:
  - {name: f, kind: ForkNode}
  - {name: m, kind: MergeNode}
successionConnections:
  - {after: start, before: f}
  - {after: f, before: a}
  - {after: f, before: b}
  - {after: a, before: m}
  - {after: b, before: m}
  - {after: m, before: tail}
",
        )],
        "M",
    );
    let r = run(&p);
    assert_eq!(r.status(), Status::Completed, "{:?}", r.error());
    assert_eq!(num(&r, "n"), "22", "UML: a merge is not a join, so `tail` runs for each of the two tokens");
}

// ── loops built from successions ─────────────────────────────────────────────

#[test]
fn a_back_edge_loop_through_a_decision_counts_to_five() {
    let p = program(
        &[(
            "L",
            "variables:
  - {name: i, value: \"0\"}
  - {name: sum, value: \"0\"}
subActions:
  - {name: body, kind: AssignmentAction, assign: \"i := i + 1; sum := sum + i\"}
  - {name: done, kind: AssignmentAction, assign: \"sum := sum + 1000\"}
controlNodes:
  - {name: m, kind: MergeNode}
  - {name: d, kind: DecisionNode}
successionConnections:
  - {after: start, before: m}
  - {after: m, before: body}
  - {after: body, before: d}
  - {after: d, before: m, guard: \"i < 5\"}
  - {after: d, before: done, guard: else}
",
        )],
        "L",
    );
    let r = run(&p);
    assert_eq!(r.status(), Status::Completed, "{:?}", r.error());
    assert_eq!(num(&r, "sum"), "1015");
}

#[test]
fn an_unbounded_back_edge_loop_is_stopped_with_an_error_not_a_hang() {
    let p = program(
        &[(
            "U",
            "variables:\n  - {name: i, value: \"0\"}\nsubActions:\n  - {name: body, kind: AssignmentAction, assign: \"i := i + 1\"}\ncontrolNodes:\n  - {name: m, kind: MergeNode}\nsuccessionConnections:\n  - {after: start, before: m}\n  - {after: m, before: body}\n  - {after: body, before: m}\n",
        )],
        "U",
    );
    let r = run(&p);
    assert_eq!(r.status(), Status::Error, "an endless loop must be bounded");
}

fn counting_loop(n: u32) -> ActionProgram {
    program(
        &[(
            "B",
            &format!("variables:\n  - {{name: i, value: \"0\"}}\nsubActions:\n  - name: w\n    kind: LoopAction\n    loopKind: while\n    condition: \"i < {n}\"\n    body:\n      - {{name: inc, kind: AssignmentAction, assign: \"i := i + 1\"}}\n"),
        )],
        "B",
    )
}

#[test]
fn the_documented_loop_limit_is_exactly_one_thousand_iterations() {
    let ok = counting_loop(1000);
    let r = run(&ok);
    assert_eq!(r.status(), Status::Completed, "{:?}", r.error());
    assert_eq!(num(&r, "i"), "1000");
    let over = counting_loop(1001);
    let r = run(&over);
    assert_eq!(r.status(), Status::Error, "1001 iterations exceed the limit");
}

#[test]
fn loops_nested_three_deep_multiply_their_counts() {
    let p = program(
        &[(
            "N",
            "variables:
  - {name: i, value: \"0\"}
  - {name: j, value: \"0\"}
  - {name: k, value: \"0\"}
  - {name: hits, value: \"0\"}
subActions:
  - name: li
    kind: LoopAction
    loopKind: while
    condition: \"i < 4\"
    body:
      - {name: ri, kind: AssignmentAction, assign: \"i := i + 1; j := 0\"}
      - name: lj
        kind: LoopAction
        loopKind: while
        condition: \"j < 3\"
        body:
          - {name: rj, kind: AssignmentAction, assign: \"j := j + 1; k := 0\"}
          - name: lk
            kind: LoopAction
            loopKind: while
            condition: \"k < 2\"
            body:
              - {name: hit, kind: AssignmentAction, assign: \"k := k + 1; hits := hits + 1\"}
successionConnections: []
",
        )],
        "N",
    );
    let r = run(&p);
    assert_eq!(r.status(), Status::Completed, "{:?}", r.error());
    assert_eq!(num(&r, "hits"), "24");
}

#[test]
fn if_inside_a_loop_inside_an_if_picks_the_right_branch_every_iteration() {
    let p = program(
        &[(
            "I",
            "variables:
  - {name: i, value: \"0\"}
  - {name: even, value: \"0\"}
  - {name: odd, value: \"0\"}
  - {name: gate, value: \"1\"}
subActions:
  - name: outer
    kind: IfAction
    condition: \"gate == 1\"
    then:
      - name: lp
        kind: LoopAction
        loopKind: while
        condition: \"i < 10\"
        body:
          - {name: step, kind: AssignmentAction, assign: \"i := i + 1\"}
          - name: parity
            kind: IfAction
            condition: \"i % 2 == 0\"
            then:
              - {name: e, kind: AssignmentAction, assign: \"even := even + 1\"}
            else:
              - {name: o, kind: AssignmentAction, assign: \"odd := odd + 1\"}
    else:
      - {name: skip, kind: AssignmentAction, assign: \"i := 99\"}
",
        )],
        "I",
    );
    let r = run(&p);
    assert_eq!(r.status(), Status::Completed, "{:?}", r.error());
    assert_eq!((num(&r, "even"), num(&r, "odd"), num(&r, "i")), ("5".into(), "5".into(), "10".into()));
}

// ── accepts and time in parallel branches ────────────────────────────────────

const TWO_ACCEPTS: &str = "variables:
  - {name: x, value: \"0\"}
  - {name: y, value: \"0\"}
  - {name: both, value: \"0\"}
subActions:
  - {name: wa, kind: AcceptAction, payload: Pkg::A, as: ea}
  - {name: wb, kind: AcceptAction, payload: Pkg::B, as: eb}
  - {name: ta, kind: AssignmentAction, assign: \"x := ea.v\"}
  - {name: tb, kind: AssignmentAction, assign: \"y := eb.v\"}
  - {name: fin, kind: AssignmentAction, assign: \"both := x * 10 + y\"}
controlNodes:
  - {name: f, kind: ForkNode}
  - {name: j, kind: JoinNode}
successionConnections:
  - {after: start, before: f}
  - {after: f, before: wa}
  - {after: f, before: wb}
  - {after: wa, before: ta}
  - {after: wb, before: tb}
  - {after: ta, before: j}
  - {after: tb, before: j}
  - {after: j, before: fin}
";

#[test]
fn two_accepts_in_parallel_branches_complete_in_either_order() {
    let p = program(&[("P", TWO_ACCEPTS)], "P");
    for (first, second) in [(("Pkg::A", "3"), ("Pkg::B", "4")), (("Pkg::B", "4"), ("Pkg::A", "3"))] {
        let mut r = run(&p);
        assert_eq!(r.status(), Status::Waiting);
        r.send(first.0, &[("v", val(first.1))]);
        assert_eq!(r.status(), Status::Waiting, "the join still waits for the other branch");
        assert!(!r.ran("fin"));
        r.send(second.0, &[("v", val(second.1))]);
        assert_eq!(r.status(), Status::Completed, "{:?}", r.error());
        assert_eq!(num(&r, "both"), "34");
    }
}

#[test]
fn an_event_for_an_already_consumed_accept_is_ignored_and_does_not_corrupt_the_join() {
    let p = program(&[("P", TWO_ACCEPTS)], "P");
    let mut r = run(&p);
    r.send("Pkg::A", &[("v", val("3"))]);
    r.send("Pkg::A", &[("v", val("9"))]);
    assert_eq!(num(&r, "x"), "3", "the second A finds nobody waiting");
    r.send("Pkg::B", &[("v", val("4"))]);
    assert_eq!(r.status(), Status::Completed);
    assert_eq!(num(&r, "both"), "34");
}

#[test]
fn timed_accepts_in_parallel_branches_fire_on_their_own_instants() {
    let p = program(
        &[(
            "T",
            "variables:
  - {name: log, value: \"0\"}
subActions:
  - {name: w1, kind: AcceptAction, after: \"1 s\"}
  - {name: w3, kind: AcceptAction, after: \"3 s\"}
  - {name: a, kind: AssignmentAction, assign: \"log := log * 10 + 1\"}
  - {name: b, kind: AssignmentAction, assign: \"log := log * 10 + 3\"}
  - {name: fin, kind: AssignmentAction, assign: \"log := log * 10 + 9\"}
controlNodes:
  - {name: f, kind: ForkNode}
  - {name: j, kind: JoinNode}
successionConnections:
  - {after: start, before: f}
  - {after: f, before: w3}
  - {after: f, before: w1}
  - {after: w1, before: a}
  - {after: w3, before: b}
  - {after: a, before: j}
  - {after: b, before: j}
  - {after: j, before: fin}
",
        )],
        "T",
    );
    let mut r = run(&p);
    r.advance(1.5);
    assert_eq!(num(&r, "log"), "1", "only the 1 s branch fired, although the 3 s branch is declared first");
    r.advance(2.0);
    assert_eq!(r.status(), Status::Completed, "{:?}", r.error());
    assert_eq!(num(&r, "log"), "139");
}

#[test]
fn one_large_advance_fires_every_due_timer_in_time_order() {
    let p = program(
        &[(
            "T",
            "variables:
  - {name: log, value: \"0\"}
subActions:
  - {name: w1, kind: AcceptAction, after: \"1 s\"}
  - {name: w3, kind: AcceptAction, after: \"3 s\"}
  - {name: a, kind: AssignmentAction, assign: \"log := log * 10 + 1\"}
  - {name: b, kind: AssignmentAction, assign: \"log := log * 10 + 3\"}
controlNodes:
  - {name: f, kind: ForkNode}
successionConnections:
  - {after: start, before: f}
  - {after: f, before: w3}
  - {after: f, before: w1}
  - {after: w1, before: a}
  - {after: w3, before: b}
",
        )],
        "T",
    );
    let mut r = run(&p);
    r.advance(10.0);
    assert_eq!(num(&r, "log"), "13", "time order, not document order");
}

// ── terminate ────────────────────────────────────────────────────────────────

#[test]
fn terminate_in_one_branch_ends_the_run_even_while_another_branch_waits() {
    let p = program(
        &[(
            "K",
            "variables:
  - {name: n, value: \"0\"}
subActions:
  - {name: w, kind: AcceptAction, payload: Pkg::Never}
  - {name: after_w, kind: AssignmentAction, assign: \"n := 1\"}
  - {name: stop, kind: TerminateAction}
controlNodes:
  - {name: f, kind: ForkNode}
successionConnections:
  - {after: start, before: f}
  - {after: f, before: w}
  - {after: f, before: stop}
  - {after: w, before: after_w}
",
        )],
        "K",
    );
    let mut r = run(&p);
    assert_eq!(r.status(), Status::Terminated);
    r.send("Pkg::Never", &[]);
    assert_eq!(r.status(), Status::Terminated, "a terminated run takes no more events");
    assert!(!r.ran("after_w"));
}

// ── calls and data ───────────────────────────────────────────────────────────

#[test]
fn a_five_deep_call_chain_threads_data_through_parameters() {
    let step = |inner: &str| {
        format!(
            "parameters:\n  - {{name: x, direction: in}}\n  - {{name: y, direction: out}}\nsubActions:\n  - {{name: pre, kind: AssignmentAction, assign: \"y := x + 1\"}}\n  - name: c\n    kind: PerformAction\n    typedBy: {inner}\n    parameters: [{{name: x, value: x}}]\n  - {{name: post, kind: AssignmentAction, assign: \"y := c.y * 2\"}}\nsuccessionConnections:\n  - {{after: pre, before: c}}\n  - {{after: c, before: post}}\n"
        )
    };
    let leaf = "parameters:\n  - {name: x, direction: in}\n  - {name: y, direction: out}\nsubActions:\n  - {name: calc, kind: AssignmentAction, assign: \"y := x + 1\"}\n";
    let c1 = step("L2");
    let c2 = step("L3");
    let c3 = step("L4");
    let c4 = step("L5");
    let p = program(&[("L1", &c1), ("L2", &c2), ("L3", &c3), ("L4", &c4), ("L5", leaf)], "L1");
    let mut r = ActRun::new(&p, ActOptions::default());
    r.set_input("x", val("1")).unwrap();
    r.run();
    assert_eq!(r.status(), Status::Completed, "{:?}", r.error());
    // L5: 2; L4: 2*2=4; L3: 8; L2: 16; L1: 32
    assert_eq!(format_value(&r.outputs().iter().find(|(n, _)| n == "y").unwrap().1), "32");
}

#[test]
fn the_same_action_performed_twice_in_a_loop_gets_fresh_outputs_each_time() {
    let b = "parameters:\n  - {name: x, direction: in}\n  - {name: y, direction: out}\nsubActions:\n  - {name: calc, kind: AssignmentAction, assign: \"y := x * 2\"}\n";
    let a = "variables:\n  - {name: i, value: \"1\"}\n  - {name: total, value: \"0\"}\nsubActions:\n  - name: lp\n    kind: LoopAction\n    loopKind: while\n    condition: \"i <= 4\"\n    body:\n      - name: call\n        kind: PerformAction\n        typedBy: B\n        parameters: [{name: x, value: i}]\n      - {name: acc, kind: AssignmentAction, assign: \"total := total + call.y; i := i + 1\"}\n";
    let p = program(&[("A", a), ("B", b)], "A");
    let r = run(&p);
    assert_eq!(r.status(), Status::Completed, "{:?}", r.error());
    assert_eq!(num(&r, "total"), "20", "2+4+6+8");
}

#[test]
fn sends_inside_a_loop_are_recorded_in_order_with_per_iteration_payloads() {
    let p = program(
        &[(
            "S",
            "variables:\n  - {name: i, value: \"0\"}\nsubActions:\n  - name: lp\n    kind: LoopAction\n    loopKind: while\n    condition: \"i < 3\"\n    body:\n      - {name: inc, kind: AssignmentAction, assign: \"i := i + 1\"}\n      - {name: out, kind: SendAction, payload: Pkg::Tick, parameters: [{name: n, value: i}]}\n",
        )],
        "S",
    );
    let r = run(&p);
    let got: Vec<_> = r.sent().iter().map(|s| s.args[0].1.clone()).collect();
    assert_eq!(got, ["1", "2", "3"]);
}

#[test]
fn unit_arithmetic_survives_a_loop_and_a_dimension_error_is_caught() {
    let ok = program(
        &[("U", "variables:\n  - {name: d, value: \"0 m\"}\n  - {name: i, value: \"0\"}\nsubActions:\n  - name: lp\n    kind: LoopAction\n    loopKind: while\n    condition: \"i < 3\"\n    body:\n      - {name: s, kind: AssignmentAction, assign: \"d := d + 2 m; i := i + 1\"}\n")],
        "U",
    );
    let r = run(&ok);
    assert_eq!(r.status(), Status::Completed, "{:?}", r.error());
    assert_eq!(num(&r, "d"), format_value(&val("6 m")));
    // metres plus seconds is found statically (E935), before any run
    static N: AtomicU64 = AtomicU64::new(1000);
    let dir: PathBuf = std::env::temp_dir().join(format!("syscribe-actstress-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(dir.join("U.md"), "---\ntype: ActionDef\nname: U\nvariables:\n  - {name: d, value: \"0 m\"}\nsubActions:\n  - {name: s, kind: AssignmentAction, assign: \"d := d + 2 s\"}\n---\n").unwrap();
    let err = lower_action(&walk_model(&dir).unwrap(), "U").unwrap_err();
    assert!(err.iter().any(|(c, _)| *c == "E935"), "{err:?}");
}

#[test]
fn a_decision_with_no_matching_guard_and_no_else_does_not_complete_silently() {
    let p = program(
        &[(
            "D",
            "variables:\n  - {name: v, value: \"5\"}\nsubActions:\n  - {name: a, kind: AssignmentAction, assign: \"v := 1\"}\ncontrolNodes:\n  - {name: d, kind: DecisionNode}\nsuccessionConnections:\n  - {after: start, before: d}\n  - {after: d, before: a, guard: \"v > 100\"}\n",
        )],
        "D",
    );
    let r = run(&p);
    assert!(!r.ran("a"));
    assert_ne!(r.status(), Status::Waiting);
    eprintln!("status with no matching guard and no else: {:?}", r.status());
}

#[test]
fn a_rerun_of_a_complex_action_gives_an_identical_trace() {
    let p = program(&[("P", TWO_ACCEPTS)], "P");
    let go = || {
        let mut r = run(&p);
        r.send("Pkg::B", &[("v", val("4"))]);
        r.send("Pkg::A", &[("v", val("3"))]);
        order(&r)
    };
    assert_eq!(go(), go());
}

#[test]
fn a_send_in_a_performed_callee_is_recorded_by_the_caller_too() {
    let p = program(
        &[
            ("Inner", "subActions:\n  - {name: s, kind: SendAction, payload: Pkg::FromCallee, parameters: [{name: n, value: \"7\"}]}\n"),
            ("Outer", "subActions:\n  - {name: first, kind: SendAction, payload: Pkg::First}\n  - {name: call, kind: PerformAction, typedBy: Inner}\nsuccessionConnections:\n  - {after: first, before: call}\n"),
        ],
        "Outer",
    );
    let r = run(&p);
    assert_eq!(r.status(), Status::Completed, "{:?}", r.error());
    let got: Vec<&str> = r.sent().iter().map(|s| s.event.as_str()).collect();
    assert_eq!(got, ["Pkg::First", "Pkg::FromCallee"]);
}

