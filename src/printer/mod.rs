use anyhow::Result;
use colored::Colorize;
use std::cell::RefCell;
use std::io;

mod formatter;

use crate::types::result::TestReport;
use formatter::{DefaultFormatter, Formatter};

#[mockall::automock]
pub trait Printer {
    fn print_report(&self, report: &TestReport) -> Result<()>;
}

#[derive(Default)]
pub struct ColorPrinter<F: Formatter, W: io::Write> {
    formatter: F,
    writer: RefCell<W>,
}

impl ColorPrinter<DefaultFormatter, io::Stdout> {
    pub fn new() -> Self {
        Self::with_dependencies(DefaultFormatter::new(), io::stdout())
    }
}

impl<F: Formatter, W: io::Write> ColorPrinter<F, W> {
    pub fn with_dependencies(formatter: F, writer: W) -> Self {
        ColorPrinter {
            formatter,
            writer: RefCell::new(writer),
        }
    }
}

impl<F: Formatter, W: io::Write> Printer for ColorPrinter<F, W> {
    fn print_report(&self, report: &TestReport) -> Result<()> {
        if report.testsuites.is_empty() {
            writeln!(self.writer.borrow_mut(), "No test case found. Exiting.")?;
            return Ok(());
        }
        let mut failures = vec![];

        for suite_result in &report.testsuites {
            writeln!(
                self.writer.borrow_mut(),
                "Running tests for {}",
                suite_result.name
            )?;
            for test_result in &suite_result.results {
                let successful = test_result.is_successful();
                let success_msg = match successful {
                    true => "OK".green(),
                    false => "FAILED".red(),
                };
                writeln!(
                    self.writer.borrow_mut(),
                    "test {} ... {}",
                    test_result.name,
                    success_msg
                )?;
                if !successful {
                    failures.push(test_result);
                }
            }
        }

        if !failures.is_empty() {
            writeln!(self.writer.borrow_mut(), "\nfailures:\n")?;
            for failure in failures {
                writeln!(self.writer.borrow_mut(), "---- {} ----", failure.name)?;
                writeln!(
                    self.writer.borrow_mut(),
                    "{}",
                    self.formatter.compute_summary(failure)
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::formatdoc;
    use mockall::*;
    use std::any::TypeId;

    use crate::types::result::{TestResult, TestResultBuilder, TestSuiteResult};

    fn type_of<T: 'static>(_: T) -> TypeId {
        TypeId::of::<T>()
    }

    #[test]
    fn test_new_calls_with_dependencies() {
        // GIVEN

        // WHEN
        let printer = ColorPrinter::new();

        // THEN
        assert_eq!(TypeId::of::<DefaultFormatter>(), type_of(printer.formatter));
        assert_eq!(
            TypeId::of::<io::Stdout>(),
            type_of(printer.writer.into_inner())
        );
    }

    #[test]
    fn test_print_report_empty() {
        // GIVEN
        let report = TestReport { testsuites: vec![] };
        let result = Vec::new();
        let printer = ColorPrinter::with_dependencies(DefaultFormatter::new(), result);

        // WHEN
        printer.print_report(&report).unwrap();

        // THEN
        assert_eq!(
            "No test case found. Exiting.\n",
            std::str::from_utf8(&printer.writer.borrow()).unwrap()
        );
    }

    #[test]
    fn test_print_report_successful_test() {
        // GIVEN
        let report = TestReport {
            testsuites: vec![TestSuiteResult {
                name: "mysuite".to_string(),
                results: vec![TestResultBuilder::default()
                    .name("mytestcase")
                    .build()
                    .unwrap()],
            }],
        };
        let result = Vec::new();
        let printer = ColorPrinter::with_dependencies(DefaultFormatter::new(), result);

        // WHEN
        printer.print_report(&report).unwrap();

        // THEN
        assert_eq!(
            formatdoc! {r#"
            Running tests for mysuite
            test mytestcase ... {ok}
            "#, ok="OK".green()},
            std::str::from_utf8(&printer.writer.borrow()).unwrap()
        );
    }

    #[test]
    fn test_print_report_failing_test() {
        // GIVEN
        let result = Vec::new();
        let (report, failing_test) = a_report_with_a_failing_test();

        let mut mock_formatter = formatter::MockFormatter::new();
        mock_formatter
            .expect_compute_summary()
            .with(predicate::eq(failing_test))
            .times(1)
            .return_const("FAIL\n");

        let printer = ColorPrinter::with_dependencies(mock_formatter, result);

        // WHEN
        printer.print_report(&report).unwrap();

        // THEN
        pretty_assertions::assert_eq!(
            formatdoc! {r#"
            Running tests for mysuite
            test failingtest ... {failed}

            failures:

            ---- failingtest ----
            FAIL

            "#, failed="FAILED".red()},
            std::str::from_utf8(&printer.writer.borrow()).unwrap()
        );
    }

    fn a_report_with_a_failing_test() -> (TestReport, TestResult) {
        let failing_test = TestResultBuilder::default()
            .name("failingtest")
            .expected_stdout("foo")
            .actual_stdout("bar")
            .build()
            .unwrap();

        let report = TestReport {
            testsuites: vec![TestSuiteResult {
                name: "mysuite".to_string(),
                results: vec![failing_test.clone()],
            }],
        };
        (report, failing_test)
    }
}
