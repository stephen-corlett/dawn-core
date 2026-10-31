//! Requires tests to use `#[spec]`, which enforces Given, When, Then at compile
//! time. See docs/architecture.md.
//!
//! This reads source text, so it can't see tests that a macro builds from parts.
//! Code review has to catch those.

/// Returns a problem for every test in a Rust file that bypasses `#[spec]`.
pub fn problems(source: &str) -> Vec<String> {
    let code = strip_strings_and_comments(source);
    let mut found: Vec<String> = attributes(&code)
        .into_iter()
        .filter(|attribute| declares_test(&attribute.text))
        .map(|attribute| {
            format!(
                "line {}: use #[spec] instead of #[{}]",
                attribute.line, attribute.text
            )
        })
        .collect();
    found.extend(doc_test_problems(source));
    found
}

/// Returns a problem if a `Cargo.toml` adds a test target that skips `#[test]`.
pub fn manifest_problems(manifest: &str) -> Vec<String> {
    manifest
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let compact: String = line.split_whitespace().collect();
            compact.starts_with("harness=false")
        })
        .map(|(index, _)| format!("line {}: harness = false isn't allowed", index + 1))
        .collect()
}

struct Attribute {
    line: usize,
    text: String,
}

/// Blanks strings and comments but keeps line breaks, so line numbers stay right.
fn strip_strings_and_comments(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut output = String::with_capacity(source.len());
    let mut index = 0;
    while index < chars.len() {
        let rest = &chars[index..];
        let end = match rest {
            ['/', '/', ..] => skip_line_comment(rest),
            ['/', '*', ..] => skip_block_comment(rest),
            ['"', ..] => skip_string(rest),
            ['r', '"' | '#', ..] => skip_raw_string(rest),
            ['\'', ..] => skip_char_literal(rest),
            [character, ..] => {
                output.push(*character);
                index += 1;
                continue;
            }
            [] => break,
        };
        output.extend(rest[..end].iter().filter(|character| **character == '\n'));
        index += end;
    }
    output
}

fn skip_line_comment(rest: &[char]) -> usize {
    rest.iter()
        .position(|character| *character == '\n')
        .unwrap_or(rest.len())
}

fn skip_block_comment(rest: &[char]) -> usize {
    let mut depth = 0;
    let mut index = 0;
    while index < rest.len() {
        match rest[index..] {
            ['/', '*', ..] => {
                depth += 1;
                index += 2;
            }
            ['*', '/', ..] => {
                depth -= 1;
                index += 2;
                if depth == 0 {
                    return index;
                }
            }
            _ => index += 1,
        }
    }
    rest.len()
}

fn skip_string(rest: &[char]) -> usize {
    let mut index = 1;
    while index < rest.len() {
        match rest[index] {
            '\\' => index += 2,
            '"' => return index + 1,
            _ => index += 1,
        }
    }
    rest.len()
}

/// Falls back to skipping only the `r` when this turns out to be an identifier.
fn skip_raw_string(rest: &[char]) -> usize {
    let hashes = rest[1..].iter().take_while(|c| **c == '#').count();
    if rest.get(hashes + 1) != Some(&'"') {
        return 1;
    }
    let start = hashes + 2;
    let closes = |index: usize| {
        rest[index] == '"'
            && rest[index + 1..]
                .iter()
                .take(hashes)
                .filter(|c| **c == '#')
                .count()
                == hashes
    };
    (start..rest.len())
        .find(|index| closes(*index))
        .map_or(rest.len(), |index| index + hashes + 1)
}

/// Lifetimes like `'a` also start with a quote, so only skip real character literals.
fn skip_char_literal(rest: &[char]) -> usize {
    match rest {
        ['\'', '\\', _, '\'', ..] => 4,
        ['\'', _, '\'', ..] => 3,
        _ => 1,
    }
}

/// Finds every `#[...]` and `#![...]`, including those spanning several lines.
fn attributes(code: &str) -> Vec<Attribute> {
    let chars: Vec<char> = code.chars().collect();
    let mut found = Vec::new();
    let mut line = 1;
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '\n' {
            line += 1;
        }
        let open = match chars[index..] {
            ['#', '[', ..] => Some(index + 1),
            ['#', '!', '[', ..] => Some(index + 2),
            _ => None,
        };
        if let Some(open) = open {
            let close = closing_bracket(&chars, open);
            found.push(Attribute {
                line,
                text: chars[open + 1..close].iter().collect(),
            });
        }
        index += 1;
    }
    found
}

fn closing_bracket(chars: &[char], open: usize) -> usize {
    let mut depth = 0;
    for (index, character) in chars.iter().enumerate().skip(open) {
        match character {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return index;
                }
            }
            _ => {}
        }
    }
    chars.len()
}

/// `cfg(test)` only limits when code compiles, so it's allowed. So is the first
/// argument of `cfg_attr`, which is that same kind of condition.
fn declares_test(attribute: &str) -> bool {
    let compact: String = attribute.split_whitespace().collect();
    let checked = if compact.starts_with("cfg(") {
        ""
    } else if let Some(arguments) = compact.strip_prefix("cfg_attr(") {
        arguments.split_once(',').map_or("", |(_, rest)| rest)
    } else {
        &compact
    };
    checked
        .split(|character: char| !character.is_alphanumeric() && character != '_')
        .any(is_test_word)
}

fn is_test_word(word: &str) -> bool {
    word.ends_with("test") || word.starts_with("test_")
}

fn doc_test_problems(source: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut in_fence = false;
    for (index, line) in source.lines().enumerate() {
        let line = line.trim();
        let Some(text) = line.strip_prefix("///").or(line.strip_prefix("//!")) else {
            in_fence = false;
            continue;
        };
        let Some(language) = text.trim().strip_prefix("```") else {
            continue;
        };
        if !in_fence && is_runnable(language) {
            found.push(format!(
                "line {}: doc tests aren't allowed, so mark this code block `ignore` or `text`",
                index + 1
            ));
        }
        in_fence = !in_fence;
    }
    found
}

fn is_runnable(language: &str) -> bool {
    language
        .split([',', ' '])
        .filter(|tag| !tag.is_empty())
        .all(|tag| matches!(tag, "rust" | "should_panic" | "no_run" | "compile_fail"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;

    fn assert_rejected(source: &str) {
        assert_eq!(problems(source).len(), 1, "{source}");
    }

    fn assert_allowed(source: &str) {
        assert_eq!(problems(source), Vec::<String>::new(), "{source}");
    }

    #[spec]
    fn test_format() {
        given("a file with a plain test", || {
            let source = "use a;\n\n#[test]\nfn x() {}\n";

            when("it's checked", || {
                let found = problems(source);

                test("the line is reported", || {
                    assert_eq!(found, ["line 3: use #[spec] instead of #[test]"]);
                });
            });
        });

        given("attributes that declare tests", || {
            let sources = [
                "#[test] // note\nfn x() {}",
                "#[test] fn x() {}",
                "#[ test ]\nfn x() {}",
                "#[\n    test\n]\nfn x() {}",
                "#[cfg_attr(unix, test)]\nfn x() {}",
                "#[tokio::test]\nfn x() {}",
                "#[bevy::test]\nfn x() {}",
                "#[rstest]\nfn x() {}",
                "#[test_case(1)]\nfn x() {}",
                "macro_rules! m { () => { #[test] fn x() {} } }",
            ];

            when("each one is checked", || {
                let rejected = |source: &str| problems(source).len() == 1;

                test("every one is rejected", || {
                    assert!(sources.into_iter().all(rejected));
                });
            });
        });

        given("attributes that only limit compilation", || {
            let sources = [
                "#[cfg(test)]\nmod tests;",
                "#[cfg(not(test))]\nfn x() {}",
                "#[cfg_attr(test, derive(Debug))]\nstruct X;",
                "#[spec]\nfn x() {}",
            ];

            when("each one is checked", || {
                let allowed = |source: &str| problems(source).is_empty();

                test("every one is allowed", || {
                    assert!(sources.into_iter().all(allowed));
                });
            });
        });

        given("test attributes that only appear in text", || {
            let sources = [
                "let a = \"#[test]\";",
                "let a = r#\"#[test]\"#;",
                "// #[test]\nfn x() {}",
                "/* #[test] */\nfn x() {}",
            ];

            when("each one is checked", || {
                let allowed = |source: &str| problems(source).is_empty();

                test("every one is allowed", || {
                    assert!(sources.into_iter().all(allowed));
                });
            });
        });

        given("a runnable doc test", || {
            let source = "/// ```\n/// assert!(true);\n/// ```\nfn x() {}";

            when("it's checked", || {
                test("it's rejected", || {
                    assert_rejected(source);
                });
            });
        });

        given("a doc test marked as rust", || {
            let source = "//! ```rust\n//! assert!(true);\n//! ```\n";

            when("it's checked", || {
                test("it's rejected", || {
                    assert_rejected(source);
                });
            });
        });

        given("a doc code block marked ignore", || {
            let source = "/// ```ignore\n/// code\n/// ```\nfn x() {}";

            when("it's checked", || {
                test("it's allowed", || {
                    assert_allowed(source);
                });
            });
        });

        given("a doc code block marked text", || {
            let source = "/// ```text\n/// output\n/// ```\nfn x() {}";

            when("it's checked", || {
                test("it's allowed", || {
                    assert_allowed(source);
                });
            });
        });

        given("a manifest with a custom test harness", || {
            let manifest = "[[test]]\nname = \"a\"\nharness = false\n";

            when("it's checked", || {
                let found = manifest_problems(manifest);

                test("the line is reported", || {
                    assert_eq!(found, ["line 3: harness = false isn't allowed"]);
                });
            });
        });

        given("a manifest with a normal test target", || {
            let manifest = "[[test]]\nname = \"a\"\n";

            when("it's checked", || {
                let found = manifest_problems(manifest);

                test("nothing is reported", || {
                    assert!(found.is_empty());
                });
            });
        });
    }
}
