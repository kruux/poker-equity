//! Runs the conformance fixture, which is the grammar's source of truth.
//!
//! Every rule of the notation has a row. The fixture is a flat file rather
//! than Rust so that a second implementation of the grammar can be held to
//! the same rows, and so a change that lands on only one side shows up here
//! as a failure rather than as a disagreement in the field months later.

use crate::notation::{parse_board, parse_dead, parse_hand, parse_hand_up_to};

/// The fixture, compiled in so the test needs no working directory.
const FIXTURE: &str = include_str!("../../../tests/fixtures/notation.tsv");

/// One row: what to read, how, and what should come back.
struct Row<'a> {
    line: usize,
    field: &'a str,
    arg: &'a str,
    input: &'a str,
    expect: &'a str,
    result: &'a str,
}

/// Reads the fixture, skipping comments and blank lines.
fn rows() -> Vec<Row<'static>> {
    FIXTURE
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim_start().starts_with('#') && !line.trim().is_empty())
        .map(|(i, line)| {
            let columns: Vec<&str> = line.split('\t').collect();
            assert_eq!(
                columns.len(),
                5,
                "line {} has {} columns, expected 5: {:?}",
                i + 1,
                columns.len(),
                line
            );
            Row {
                line: i + 1,
                field: columns[0],
                arg: columns[1],
                input: columns[2],
                expect: columns[3],
                result: columns[4],
            }
        })
        .collect()
}

/// Reads one row's input, returning the canonical rendering or the error's
/// stable name.
fn run(row: &Row) -> Result<String, String> {
    match row.field {
        "hand" => {
            let slots: usize = row.arg.parse().expect("hand rows carry a slot count");
            parse_hand(row.input, slots)
                .map(|spec| spec.to_string())
                .map_err(|error| error.kind.name().to_string())
        }
        "hand_up_to" => {
            let slots: usize = row.arg.parse().expect("hand_up_to rows carry a slot count");
            parse_hand_up_to(row.input, slots)
                .map(|spec| spec.to_string())
                .map_err(|error| error.kind.name().to_string())
        }
        "board" => {
            let limit: usize = row.arg.parse().expect("board rows carry a limit");
            parse_board(row.input, limit)
                .map(|slots| {
                    slots
                        .iter()
                        .map(|slot| {
                            crate::notation::HandSpec::from_slots(&[*slot]).to_string()
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .map_err(|error| error.kind.name().to_string())
        }
        "dead" => parse_dead(row.input)
            .map(|set| set.to_string())
            .map_err(|error| error.kind.name().to_string()),
        other => panic!("line {}: unknown field kind {:?}", row.line, other),
    }
}

#[test]
fn test_every_fixture_row() {
    let mut failures = Vec::new();

    for row in rows() {
        let outcome = run(&row);
        let matched = match (row.expect, &outcome) {
            ("ok", Ok(rendered)) => rendered == row.result,
            ("err", Err(name)) => name == row.result,
            _ => false,
        };
        if !matched {
            failures.push(format!(
                "line {}: {} {:?}\n     expected {} {:?}\n     got      {}",
                row.line,
                row.field,
                row.input,
                row.expect,
                row.result,
                match &outcome {
                    Ok(rendered) => format!("ok {:?}", rendered),
                    Err(name) => format!("err {:?}", name),
                }
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of the fixture's rows disagree with the parser:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// A row that reads cleanly must render to something that reads back the
/// same way, or the canonical form is not canonical.
#[test]
fn test_canonical_output_round_trips() {
    let mut failures = Vec::new();

    for row in rows().iter().filter(|row| row.expect == "ok") {
        // Only hand rows have a rendering that is itself a hand field.
        if row.field != "hand" {
            continue;
        }
        let slots: usize = row.arg.parse().unwrap();
        let Ok(first) = parse_hand(row.input, slots) else {
            continue;
        };
        let rendered = first.to_string();
        match parse_hand(&rendered, slots) {
            Ok(second) if second == first => {}
            Ok(_) => failures.push(format!(
                "line {}: {:?} renders as {:?}, which reads back as something else",
                row.line, row.input, rendered
            )),
            Err(error) => failures.push(format!(
                "line {}: {:?} renders as {:?}, which will not read back: {}",
                row.line, row.input, rendered, error
            )),
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
