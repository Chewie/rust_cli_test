//! # Test cases
//!
//! This module contains the various structs representing the test cases that we want to run:
//!
//! * A [TestCase] represents the specification for a single test.
//! * A [TestSuite] is a collection of TestCases, typically all tests from a given YAML file.
//! * A [TestSuiteCollection] is the entirety of all the TestSuites, typically all YAML files in a
//!   directory.

pub use serde::{Deserialize, Serialize};

/// The specification for a test run.
///
/// This is usually part of a [TestSuite]
#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub cmd: String,
    #[serde(default)]
    pub stdin: String,
    #[serde(default)]
    pub stdout: String,
    #[serde(default)]
    pub stderr: String,
    #[serde(default)]
    pub status: i32,
}

/// A collection of TestCases
///
/// This is usually part of a [TestSuiteCollection]
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct TestSuite {
    pub name: String,
    pub tests: Vec<TestCase>,
}

/// A collection of TestSuites
///
/// This usually represents the entirety of your tests, for example all the YAML files in a given
/// directory.
#[derive(Eq, PartialEq, Debug, Clone, Default)]
pub struct TestSuiteCollection {
    pub testsuites: Vec<TestSuite>,
}

impl TestSuiteCollection {
    pub fn new<I>(testsuites: I) -> Self
    where
        I: IntoIterator<Item = TestSuite>,
    {
        TestSuiteCollection {
            testsuites: testsuites.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_from_testsuites() {
        // GIVEN
        let testsuites = vec![
            TestSuite {
                name: "suite1".to_string(),
                tests: vec![],
            },
            TestSuite {
                name: "suite2".to_string(),
                tests: vec![],
            },
        ];

        // WHEN
        let result = TestSuiteCollection::new(testsuites);

        // THEN
        assert_eq!(
            TestSuiteCollection {
                testsuites: vec![
                    TestSuite {
                        name: "suite1".to_string(),
                        tests: vec![],
                    },
                    TestSuite {
                        name: "suite2".to_string(),
                        tests: vec![],
                    }
                ]
            },
            result
        );
    }
}
