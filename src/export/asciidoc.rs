use super::markup::Alignment;
use crate::export::markup::MarkupExporter;

#[derive(Default)]
pub struct AsciidocExporter {}

impl MarkupExporter for AsciidocExporter {
    fn table_header(&self, cell_aligmnents: &[Alignment]) -> String {
        format!(
            "[cols=\"{}\"]\n|===",
            cell_aligmnents
                .iter()
                .map(|a| match a {
                    Alignment::Left => "<",
                    Alignment::Right => ">",
                })
                .collect::<Vec<&str>>()
                .join(",")
        )
    }

    fn table_footer(&self, _cell_aligmnents: &[Alignment]) -> String {
        "|===\n".to_string()
    }

    fn table_row(&self, cells: &[&str]) -> String {
        format!("\n| {} \n", cells.join(" \n| "))
    }

    fn table_divider(&self, _cell_aligmnents: &[Alignment]) -> String {
        "".to_string()
    }

    fn command(&self, cmd: &str) -> String {
        format!("`{cmd}`")
    }
}

/// Check Asciidoc-based data row formatting
#[test]
fn test_asciidoc_exporter_table_data() {
    let exporter = AsciidocExporter::default();
    let data = vec!["a", "b", "c"];

    let actual = exporter.table_row(&data);
    let expect = "\n| a \n| b \n| c \n";

    assert_eq!(expect, actual);
}

/// Check Asciidoc-based table header formatting
#[test]
fn test_asciidoc_exporter_table_header() {
    let exporter = AsciidocExporter::default();
    let cells_alignment = [
        Alignment::Left,
        Alignment::Right,
        Alignment::Right,
        Alignment::Right,
        Alignment::Right,
    ];

    let actual = exporter.table_header(&cells_alignment);
    let expect = "[cols=\"<,>,>,>,>\"]\n|===";

    assert_eq!(expect, actual);
}

/// Test helper function to create unit-based header and horizontal line
/// independently from the markup functionality for Asciidoc.
#[cfg(test)]
fn cfg_test_table_header(unit_short_name: &str) -> String {
    format!(
        "[cols=\"<,>,>,>,>\"]\n|===\n| Command \n| Mean [{unit_short_name}] \n| Min [{unit_short_name}] \n| Max [{unit_short_name}] \n| Relative \n"
    )
}

#[cfg(test)]
use crate::options::SortOrder;

#[cfg(test)]
use crate::util::units::Unit;

#[cfg(test)]
use crate::export::BenchmarkResult;

#[cfg(test)]
use crate::export::Exporter;

/// Integration test
#[test]
fn test_asciidoc_format_s() {
    use std::collections::BTreeMap;
    let exporter = AsciidocExporter::default();

    let results = vec![
        BenchmarkResult {
            command: String::from("FOO=1 BAR=2 command | 1"),
            command_with_unused_parameters: String::from("FOO=1 BAR=2 command | 1"),
            mean: 1.0,
            stddev: Some(2.0),
            median: 1.0,
            user: 3.0,
            system: 4.0,
            min: 5.0,
            max: 6.0,
            times: Some(vec![7.0, 8.0, 9.0]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: {
                let mut params = BTreeMap::new();
                params.insert("foo".into(), "1".into());
                params.insert("bar".into(), "2".into());
                params
            },
        },
        BenchmarkResult {
            command: String::from("FOO=1 BAR=7 command | 2"),
            command_with_unused_parameters: String::from("FOO=1 BAR=7 command | 2"),
            mean: 11.0,
            stddev: Some(12.0),
            median: 11.0,
            user: 13.0,
            system: 14.0,
            min: 15.0,
            max: 16.0,
            times: Some(vec![17.0, 18.0, 19.0]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: {
                let mut params = BTreeMap::new();
                params.insert("foo".into(), "1".into());
                params.insert("bar".into(), "7".into());
                params
            },
        },
    ];

    let actual = String::from_utf8(
        exporter
            .serialize(&results, Some(Unit::Second), SortOrder::Command)
            .unwrap(),
    )
    .unwrap();
    let expect = format!(
        "{}
| `FOO=1 BAR=2 command \\| 1` 
| 1.000 ± 2.000 
| 5.000 
| 6.000 
| 1.00 

| `FOO=1 BAR=7 command \\| 2` 
| 11.000 ± 12.000 
| 15.000 
| 16.000 
| 11.00 ± 25.06 
|===
",
        cfg_test_table_header("s")
    );

    assert_eq!(expect, actual);
}

/// This test demonstrates that the given unit (ms) is used to set
/// the units for all entries.
#[test]
fn test_asciidoc_format_ms() {
    use std::collections::BTreeMap;
    let exporter = AsciidocExporter::default();

    let results = vec![
        BenchmarkResult {
            command: String::from("FOO=1 BAR=7 command | 2"),
            command_with_unused_parameters: String::from("FOO=1 BAR=7 command | 2"),
            mean: 0.011,
            stddev: Some(0.012),
            median: 0.011,
            user: 0.013,
            system: 0.014,
            min: 0.015,
            max: 0.016,
            times: Some(vec![0.017, 0.018, 0.019]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: {
                let mut params = BTreeMap::new();
                params.insert("foo".into(), "1".into());
                params.insert("bar".into(), "7".into());
                params
            },
        },
        BenchmarkResult {
            command: String::from("FOO=1 BAR=2 command | 1"),
            command_with_unused_parameters: String::from("FOO=1 BAR=2 command | 1"),
            mean: 1.0,
            stddev: Some(2.0),
            median: 1.0,
            user: 3.0,
            system: 4.0,
            min: 5.0,
            max: 6.0,
            times: Some(vec![7.0, 8.0, 9.0]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: {
                let mut params = BTreeMap::new();
                params.insert("foo".into(), "1".into());
                params.insert("bar".into(), "2".into());
                params
            },
        },
    ];

    let actual = String::from_utf8(
        exporter
            .serialize(&results, Some(Unit::MilliSecond), SortOrder::Command)
            .unwrap(),
    )
    .unwrap();
    let expect = format!(
        "{}
| `FOO=1 BAR=7 command \\| 2` 
| 11.0 ± 12.0 
| 15.0 
| 16.0 
| 1.00 

| `FOO=1 BAR=2 command \\| 1` 
| 1000.0 ± 2000.0 
| 5000.0 
| 6000.0 
| 90.91 ± 207.11 
|===
",
        cfg_test_table_header("ms")
    );

    assert_eq!(expect, actual);
}
