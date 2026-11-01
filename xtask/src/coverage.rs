//! Finds changed lines that no unit or integration test executed. See
//! docs/architecture.md.

use std::collections::{BTreeMap, BTreeSet};

/// Paths left out of the coverage check. `xtask` is project tooling, `bdd` runs inside
/// the compiler, and the rest open windows or start whole programs.
pub const EXCLUDED_PATHS: [&str; 6] = [
    "crates/client/src/main.rs",
    "crates/client/src/view.rs",
    "crates/server/src/main.rs",
    "libs/bdd/",
    "libs/sandbox_kit/",
    "xtask/",
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
