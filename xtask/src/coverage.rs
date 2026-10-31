//! Finds changed lines that no unit or integration test executed. See
//! docs/architecture.md.

use std::collections::{BTreeMap, BTreeSet};

/// Paths left out of the coverage check because tests can't reach them. `bdd` runs
/// inside the compiler, and the rest open windows or start whole programs.
pub const EXCLUDED_PATHS: [&str; 6] = [
    "crates/client/src/main.rs",
    "crates/client/src/view.rs",
    "crates/server/src/main.rs",
    "libs/bdd/",
    "libs/sandbox_kit/",
    "xtask/src/main.rs",
];

/// The lowest share of changed executable lines that tests must run.
pub const MIN_DIFF_COVERAGE_PERCENT: f64 = 90.0;

/// Limits that keep the pull request comment short. The job summary has every line.
const COMMENT_MAX_FILES: usize = 10;
const COMMENT_MAX_RANGES: usize = 6;

/// Line numbers, per file, that a report or diff mentions.
pub type LinesByFile = BTreeMap<String, BTreeSet<usize>>;

/// How often each executable line ran, per file.
pub type CountsByFile = BTreeMap<String, BTreeMap<usize, u64>>;

pub fn is_excluded(path: &str) -> bool {
    EXCLUDED_PATHS
        .iter()
        .any(|excluded| path.starts_with(excluded))
}

/// Reads `SF:` and `DA:` records from an LCOV report. A line reported more than
/// once, such as by two binaries, adds up across those records.
pub fn parse_lcov(report: &str, root: &str) -> CountsByFile {
    let root = normalize(root);
    let mut files = CountsByFile::new();
    let mut current = None;
    for line in report.lines() {
        if let Some(path) = line.strip_prefix("SF:") {
            let path = normalize(path);
            let relative = path.strip_prefix(&root).unwrap_or(&path);
            current = Some(relative.trim_start_matches('/').to_owned());
        } else if let (Some(file), Some(data)) = (&current, line.strip_prefix("DA:")) {
            let mut fields = data.split(',');
            let line_number = fields.next().and_then(|value| value.parse().ok());
            let count = fields.next().and_then(|value| value.parse::<u64>().ok());
            if let (Some(line_number), Some(count)) = (line_number, count) {
                *files
                    .entry(file.clone())
                    .or_default()
                    .entry(line_number)
                    .or_default() += count;
            }
        }
    }
    files
}

fn normalize(path: &str) -> String {
    path.replace('\\', "/")
}

/// Reads the added lines from `git diff --unified=0`.
pub fn parse_diff(diff: &str) -> LinesByFile {
    let mut files = LinesByFile::new();
    let mut current = None;
    for line in diff.lines() {
        if let Some(path) = line.strip_prefix("+++ b/") {
            current = Some(path.to_owned());
        } else if line.starts_with("+++ ") {
            current = None;
        } else if let (Some(file), Some(hunk)) = (&current, line.strip_prefix("@@ ")) {
            let added = hunk.split_whitespace().nth(1).and_then(added_range);
            if let Some((start, count)) = added {
                files
                    .entry(file.clone())
                    .or_default()
                    .extend(start..start + count);
            }
        }
    }
    files
}

/// Reads `+start,count` from a hunk header. The count is 1 when it's left out.
fn added_range(range: &str) -> Option<(usize, usize)> {
    let range = range.strip_prefix('+')?;
    match range.split_once(',') {
        Some((start, count)) => Some((start.parse().ok()?, count.parse().ok()?)),
        None => Some((range.parse().ok()?, 1)),
    }
}

/// Returns the first line of a file's `#[cfg(test)]` block. Changed test code always
/// runs, so counting it would let new tests hide untested source lines.
pub fn test_module_start(source: &str) -> Option<usize> {
    source
        .lines()
        .position(|line| line.trim() == "#[cfg(test)]")
        .map(|index| index + 1)
}

/// How well a set of changed lines is covered.
#[derive(Debug, Default, PartialEq)]
pub struct DiffCoverage {
    pub executable_lines: usize,
    pub missing: LinesByFile,
}

impl DiffCoverage {
    pub fn missing_lines(&self) -> usize {
        self.missing.values().map(BTreeSet::len).sum()
    }

    /// A diff with nothing to execute has nothing left uncovered.
    pub fn percent(&self) -> f64 {
        if self.executable_lines == 0 {
            return 100.0;
        }
        let covered = self.executable_lines - self.missing_lines();
        covered as f64 * 100.0 / self.executable_lines as f64
    }

    pub fn passes(&self) -> bool {
        self.percent() >= MIN_DIFF_COVERAGE_PERCENT
    }

    /// The results as Markdown for a GitHub job summary.
    pub fn summary_markdown(&self) -> String {
        let status = if self.passes() { "Passed" } else { "Failed" };
        let mut text = format!(
            "## Diff coverage: {:.1}%\n\n{status}: {} of {} changed lines covered \
             (minimum {MIN_DIFF_COVERAGE_PERCENT}%).\n",
            self.percent(),
            self.executable_lines - self.missing_lines(),
            self.executable_lines,
        );
        if !self.missing.is_empty() {
            text.push_str("\n| File | Line | Suggested test level |\n|---|---|---|\n");
            for (file, line, level) in self.missing_entries() {
                text.push_str(&format!("| `{file}` | {line} | {level} |\n"));
            }
        }
        text
    }

    /// A short version of the results for a pull request comment.
    pub fn comment_markdown(&self) -> String {
        let icon = if self.passes() { "✅" } else { "❌" };
        let mut text = format!(
            "### {icon} Diff coverage: {:.1}% (minimum {MIN_DIFF_COVERAGE_PERCENT}%)\n\n",
            self.percent(),
        );
        if self.executable_lines == 0 {
            text.push_str("No executable lines changed.\n");
            return text;
        }
        text.push_str(&format!(
            "{} of {} changed lines covered.\n",
            self.executable_lines - self.missing_lines(),
            self.executable_lines,
        ));
        if self.missing.is_empty() {
            return text;
        }

        text.push_str("\n| File | Missing lines | Suggested test |\n|---|---|---|\n");
        for (file, lines) in self.missing.iter().take(COMMENT_MAX_FILES) {
            let level = suggested_level(file);
            text.push_str(&format!(
                "| `{file}` | {} | {level} |\n",
                shown_ranges(lines)
            ));
        }
        if self.missing.len() > COMMENT_MAX_FILES {
            text.push_str(&format!(
                "\n{} more files are in the job summary.\n",
                self.missing.len() - COMMENT_MAX_FILES,
            ));
        }
        text
    }

    /// One GitHub annotation per missing line, shown inline on the pull request.
    pub fn annotations(&self) -> Vec<String> {
        self.missing_entries()
            .map(|(file, line, level)| {
                format!(
                    "::warning file={file},line={line},title=Missing test coverage::\
                     No test ran this line. Suggested level: {level}"
                )
            })
            .collect()
    }

    pub fn missing_entries(&self) -> impl Iterator<Item = (&str, usize, &'static str)> {
        self.missing.iter().flat_map(|(file, lines)| {
            let level = suggested_level(file);
            lines.iter().map(move |line| (file.as_str(), *line, level))
        })
    }
}

/// Writes lines as short ranges, such as `3-5, 9, +2 more`.
fn shown_ranges(lines: &BTreeSet<usize>) -> String {
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for &line in lines {
        match ranges.last_mut() {
            Some((_, end)) if *end + 1 == line => *end = line,
            _ => ranges.push((line, line)),
        }
    }

    let hidden = ranges.len().saturating_sub(COMMENT_MAX_RANGES);
    let mut shown: Vec<String> = ranges
        .iter()
        .take(COMMENT_MAX_RANGES)
        .map(|(start, end)| {
            if start == end {
                start.to_string()
            } else {
                format!("{start}-{end}")
            }
        })
        .collect();
    if hidden > 0 {
        shown.push(format!("+{hidden} more"));
    }
    shown.join(", ")
}

/// Compares changed lines with coverage. Lines missing from the report, such as
/// blanks and comments, have nothing to run.
pub fn diff_coverage(changed: &LinesByFile, coverage: &CountsByFile) -> DiffCoverage {
    let mut result = DiffCoverage::default();
    for (file, lines) in changed {
        if is_excluded(file) || !file.ends_with(".rs") {
            continue;
        }
        let Some(counts) = coverage.get(file) else {
            continue;
        };
        let executable: Vec<usize> = lines
            .iter()
            .copied()
            .filter(|line| counts.contains_key(line))
            .collect();
        result.executable_lines += executable.len();
        let never_ran: BTreeSet<usize> = executable
            .into_iter()
            .filter(|line| counts[line] == 0)
            .collect();
        if !never_ran.is_empty() {
            result.missing.insert(file.clone(), never_ran);
        }
    }
    result
}

/// Suggests where a test belongs. The person writing it makes the final call.
pub fn suggested_level(path: &str) -> &'static str {
    if path.starts_with("crates/server/") || path == "crates/client/src/network.rs" {
        "integration"
    } else if path.starts_with("crates/client/") {
        "review"
    } else {
        "unit"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;

    fn lines(file: &str, numbers: &[usize]) -> LinesByFile {
        BTreeMap::from([(file.to_owned(), numbers.iter().copied().collect())])
    }

    fn counts(file: &str, entries: &[(usize, u64)]) -> CountsByFile {
        BTreeMap::from([(file.to_owned(), entries.iter().copied().collect())])
    }

    #[spec]
    fn coverage() {
        given("an LCOV report with a Windows path", || {
            let report = "SF:D:\\Dev\\Game\\crates\\movement\\src\\rules.rs\nDA:3,2\nDA:4,0\n";

            when("it's parsed against the workspace root", || {
                let parsed = parse_lcov(report, "D:\\Dev\\Game");

                test("paths are relative with forward slashes", || {
                    assert_eq!(
                        parsed,
                        counts("crates/movement/src/rules.rs", &[(3, 2), (4, 0)])
                    );
                });
            });
        });

        given("an LCOV line reported by two binaries", || {
            let report = "SF:/g/a.rs\nDA:3,0\nend_of_record\nSF:/g/a.rs\nDA:3,4\nend_of_record\n";

            when("it's parsed", || {
                let parsed = parse_lcov(report, "/g");

                test("the counts are added together", || {
                    assert_eq!(parsed, counts("a.rs", &[(3, 4)]));
                });
            });
        });

        given("a diff with several kinds of hunk", || {
            let diff = "+++ b/a.rs\n@@ -1,0 +2,3 @@\n@@ -9 +12 @@\n@@ -20,2 +24,0 @@\n";

            when("it's parsed", || {
                let parsed = parse_diff(diff);

                test("only added lines are kept", || {
                    assert_eq!(parsed, lines("a.rs", &[2, 3, 4, 12]));
                });
            });
        });

        given("a diff that deletes a file", || {
            let diff = "+++ /dev/null\n@@ -1,4 +0,0 @@\n";

            when("it's parsed", || {
                let parsed = parse_diff(diff);

                test("nothing is kept", || {
                    assert!(parsed.is_empty());
                });
            });
        });

        given("a changed line that never ran", || {
            let changed = lines("crates/movement/src/rules.rs", &[5]);
            let report = counts("crates/movement/src/rules.rs", &[(5, 0)]);

            when("coverage is compared", || {
                let missing = diff_coverage(&changed, &report).missing;

                test("the line is reported", || {
                    assert_eq!(missing, changed);
                });
            });
        });

        given("a changed line that a test ran", || {
            let changed = lines("crates/movement/src/rules.rs", &[5]);
            let report = counts("crates/movement/src/rules.rs", &[(5, 1)]);

            when("coverage is compared", || {
                let missing = diff_coverage(&changed, &report).missing;

                test("nothing is reported", || {
                    assert!(missing.is_empty());
                });
            });
        });

        given("a changed line with no executable code", || {
            let changed = lines("crates/movement/src/rules.rs", &[5]);
            let report = counts("crates/movement/src/rules.rs", &[(4, 0)]);

            when("coverage is compared", || {
                let missing = diff_coverage(&changed, &report).missing;

                test("nothing is reported", || {
                    assert!(missing.is_empty());
                });
            });
        });

        given("an uncovered line in excluded code", || {
            let changed = lines("crates/client/src/view.rs", &[5]);
            let report = counts("crates/client/src/view.rs", &[(5, 0)]);

            when("coverage is compared", || {
                let missing = diff_coverage(&changed, &report).missing;

                test("nothing is reported", || {
                    assert!(missing.is_empty());
                });
            });
        });

        given("nine of ten changed lines covered", || {
            let entries: Vec<(usize, u64)> =
                (1..=10).map(|line| (line, u64::from(line > 1))).collect();
            let changed = lines(
                "crates/movement/src/rules.rs",
                &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            );
            let report = counts("crates/movement/src/rules.rs", &entries);

            when("coverage is compared", || {
                let result = diff_coverage(&changed, &report);

                test("exactly 90 percent passes", || {
                    assert_eq!(result.percent(), 90.0);
                    assert!(result.passes());
                });
            });
        });

        given("eight of ten changed lines covered", || {
            let entries: Vec<(usize, u64)> =
                (1..=10).map(|line| (line, u64::from(line > 2))).collect();
            let changed = lines(
                "crates/movement/src/rules.rs",
                &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            );
            let report = counts("crates/movement/src/rules.rs", &entries);

            when("coverage is compared", || {
                let result = diff_coverage(&changed, &report);

                test("80 percent fails", || {
                    assert!(!result.passes());
                });
            });
        });

        given("a diff with no executable lines", || {
            let changed = lines("crates/movement/src/rules.rs", &[5]);
            let report = counts("crates/movement/src/rules.rs", &[(4, 0)]);

            when("coverage is compared", || {
                let result = diff_coverage(&changed, &report);

                test("it passes at 100 percent", || {
                    assert_eq!(result.percent(), 100.0);
                });
            });
        });

        given("a result with one missing line", || {
            let changed = lines("crates/movement/src/rules.rs", &[5]);
            let report = counts("crates/movement/src/rules.rs", &[(5, 0)]);

            when("it's formatted for GitHub", || {
                let result = diff_coverage(&changed, &report);
                let summary = result.summary_markdown();
                let annotations = result.annotations();

                test("the summary names the line and level", || {
                    assert!(summary.contains("## Diff coverage: 0.0%"));
                    assert!(summary.contains("| `crates/movement/src/rules.rs` | 5 | unit |"));
                });

                test("the annotation points at the line", || {
                    assert_eq!(annotations.len(), 1);
                    assert!(
                        annotations[0]
                            .starts_with("::warning file=crates/movement/src/rules.rs,line=5,")
                    );
                });
            });
        });

        given("a source file with a test module", || {
            let source = "fn a() {}\n\n#[cfg(test)]\nmod tests {}\n";

            when("its test module is located", || {
                let start = test_module_start(source);

                test("the first test line is found", || {
                    assert_eq!(start, Some(3));
                });
            });
        });

        given("a source file with no test module", || {
            let source = "fn a() {}\n";

            when("its test module is located", || {
                let start = test_module_start(source);

                test("nothing is found", || {
                    assert_eq!(start, None);
                });
            });
        });

        given("a failing result with grouped lines", || {
            let changed = lines("crates/movement/src/rules.rs", &[3, 4, 5, 9]);
            let entries = [(3, 0), (4, 0), (5, 0), (9, 0)];
            let report = counts("crates/movement/src/rules.rs", &entries);

            when("it's written as a comment", || {
                let comment = diff_coverage(&changed, &report).comment_markdown();

                test("the heading shows the failure", || {
                    assert!(comment.starts_with("### ❌ Diff coverage: 0.0% (minimum 90%)"));
                });

                test("the lines are shown as ranges", || {
                    assert!(comment.contains("| 3-5, 9 | unit |"));
                });
            });
        });

        given("a fully covered result", || {
            let changed = lines("crates/movement/src/rules.rs", &[5]);
            let report = counts("crates/movement/src/rules.rs", &[(5, 1)]);

            when("it's written as a comment", || {
                let comment = diff_coverage(&changed, &report).comment_markdown();

                test("it passes without a table", || {
                    assert!(comment.starts_with("### ✅ Diff coverage: 100.0%"));
                    assert!(comment.contains("1 of 1 changed lines covered."));
                    assert!(!comment.contains('|'));
                });
            });
        });

        given("a passing result with a missing line", || {
            let numbers: Vec<usize> = (1..=10).collect();
            let entries: Vec<(usize, u64)> =
                numbers.iter().map(|&n| (n, u64::from(n > 1))).collect();
            let changed = lines("crates/movement/src/rules.rs", &numbers);
            let report = counts("crates/movement/src/rules.rs", &entries);

            when("it's written as a comment", || {
                let comment = diff_coverage(&changed, &report).comment_markdown();

                test("it passes and still names the line", || {
                    assert!(comment.starts_with("### ✅ Diff coverage: 90.0%"));
                    assert!(comment.contains("| 1 | unit |"));
                });
            });
        });

        given("a diff with no executable lines", || {
            let result = DiffCoverage::default();

            when("it's written as a comment", || {
                let comment = result.comment_markdown();

                test("it says so", || {
                    assert!(comment.contains("No executable lines changed."));
                });
            });
        });

        given("more scattered lines than a comment should show", || {
            let scattered: BTreeSet<usize> = (1..=8).map(|line| line * 10).collect();

            when("they're written as ranges", || {
                let shown = shown_ranges(&scattered);

                test("the rest are counted", || {
                    assert_eq!(shown, "10, 20, 30, 40, 50, 60, +2 more");
                });
            });
        });

        given("more files than a comment should show", || {
            let missing: LinesByFile = (0..12)
                .map(|file| (format!("crates/a/src/f{file}.rs"), BTreeSet::from([1])))
                .collect();
            let result = DiffCoverage {
                executable_lines: 12,
                missing,
            };

            when("it's written as a comment", || {
                let comment = result.comment_markdown();
                let rows = comment.lines().filter(|line| line.contains(".rs")).count();

                test("only the limit is listed", || {
                    assert_eq!(rows, 10);
                    assert!(comment.contains("2 more files are in the job summary."));
                });
            });
        });

        given("code in a game module", || {
            let path = "crates/combat/src/rules.rs";

            when("a level is suggested", || {
                let level = suggested_level(path);

                test("it's a unit test", || {
                    assert_eq!(level, "unit");
                });
            });
        });

        given("code in the server app", || {
            let path = "crates/server/src/network.rs";

            when("a level is suggested", || {
                let level = suggested_level(path);

                test("it's an integration test", || {
                    assert_eq!(level, "integration");
                });
            });
        });

        given("the client's network code", || {
            let path = "crates/client/src/network.rs";

            when("a level is suggested", || {
                let level = suggested_level(path);

                test("it's an integration test", || {
                    assert_eq!(level, "integration");
                });
            });
        });

        given("other client code", || {
            let path = "crates/client/src/lib.rs";

            when("a level is suggested", || {
                let level = suggested_level(path);

                test("it's left for review", || {
                    assert_eq!(level, "review");
                });
            });
        });
    }
}
