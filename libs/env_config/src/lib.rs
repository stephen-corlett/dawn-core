//! Typed environment variables, inspired by Node's `envalid`.
//!
//! ```ignore
//! let mut env = Env::from_process("APP_ENV");
//! let port = env.read(
//!     "APP_PORT",
//!     parsed::<u16>().desc("Port to listen on").default(8080).dev_default(3000),
//! );
//! let key = env.read("APP_KEY", hex::<32>().desc("Signing key").dev_default([0; 32]));
//! env.validate()?;
//!
//! let port = port.value();
//! ```
//!
//! A variable with neither `default` nor `dev_default` is required. `dev_default`
//! only applies in development, and an unset environment counts as production, so a
//! misconfigured deployment fails rather than running with development values.
//! `validate` reports every problem at once.

use core::str::FromStr;
use std::collections::BTreeMap;

const DEVELOPMENT: &str = "development";
const PRODUCTION: &str = "production";

/// Variables being read, and every problem found so far.
pub struct Env {
    vars: BTreeMap<String, String>,
    is_production: bool,
    problems: Vec<String>,
}

impl Env {
    /// Reads the process's variables. `mode_var` names the variable that selects
    /// `development` or `production`.
    pub fn from_process(mode_var: &str) -> Self {
        Self::from_vars(mode_var, std::env::vars())
    }

    /// Reads supplied variables, which keeps tests away from the real environment.
    pub fn from_vars<K, V>(mode_var: &str, vars: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        let vars: BTreeMap<String, String> = vars
            .into_iter()
            .map(|(name, value)| (name.into(), value.into()))
            .filter(|(_, value)| !value.is_empty())
            .collect();
        let mode = vars.get(mode_var).map(String::as_str);
        let mut problems = Vec::new();
        if let Some(mode) = mode.filter(|mode| ![DEVELOPMENT, PRODUCTION].contains(mode)) {
            problems.push(format!(
                "{mode_var}: must be {DEVELOPMENT} or {PRODUCTION}, not {mode}"
            ));
        }
        let is_production = mode != Some(DEVELOPMENT);
        Self {
            vars,
            is_production,
            problems,
        }
    }

    /// A development environment with no variables set.
    pub fn development(mode_var: &str) -> Self {
        Self::from_vars(mode_var, [(mode_var, DEVELOPMENT)])
    }

    pub fn is_production(&self) -> bool {
        self.is_production
    }

    /// Reads one variable. Call `validate` before using the returned field.
    pub fn read<T: Clone + PartialEq>(&mut self, name: &str, spec: Spec<T>) -> Field<T> {
        match self.resolve(name, spec) {
            Ok(value) => Field(Some(value)),
            Err(problem) => {
                self.problems.push(format!("{name}: {problem}"));
                Field(None)
            }
        }
    }

    fn resolve<T: Clone + PartialEq>(&self, name: &str, spec: Spec<T>) -> Result<T, String> {
        let fallback = if self.is_production {
            spec.default.clone()
        } else {
            spec.dev_default.clone().or(spec.default.clone())
        };
        let value = match (self.vars.get(name), fallback) {
            (Some(raw), _) => (spec.parser)(raw).map_err(|problem| spec.describe(&problem))?,
            (None, Some(value)) => value,
            (None, None) => return Err(spec.describe("missing")),
        };
        if !spec.choices.is_empty() && !spec.choices.contains(&value) {
            return Err(spec.describe("not an allowed value"));
        }
        if self.is_production && spec.forbidden_in_production.as_ref() == Some(&value) {
            return Err(spec.describe("can't use this value in production"));
        }
        Ok(value)
    }

    /// Returns every problem together, so they can all be fixed in one pass.
    pub fn validate(self) -> Result<(), String> {
        if self.problems.is_empty() {
            Ok(())
        } else {
            Err(self.problems.join("\n"))
        }
    }
}

/// A value read from the environment.
pub struct Field<T>(Option<T>);

impl<T> Field<T> {
    /// # Panics
    ///
    /// Panics if `validate` wasn't called first, or if it failed.
    pub fn value(self) -> T {
        self.0
            .expect("Env::validate must pass before using a field")
    }
}

type Parser<T> = Box<dyn Fn(&str) -> Result<T, String>>;

/// How to read one variable.
pub struct Spec<T> {
    parser: Parser<T>,
    description: Option<&'static str>,
    default: Option<T>,
    dev_default: Option<T>,
    choices: Vec<T>,
    forbidden_in_production: Option<T>,
}

impl<T> Spec<T> {
    /// Describes the variable in error messages.
    pub fn desc(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }

    /// The value in every environment when the variable is unset.
    pub fn default(mut self, value: T) -> Self {
        self.default = Some(value);
        self
    }

    /// The value in development when the variable is unset.
    pub fn dev_default(mut self, value: T) -> Self {
        self.dev_default = Some(value);
        self
    }

    /// Limits the variable to these values.
    pub fn choices(mut self, values: impl IntoIterator<Item = T>) -> Self {
        self.choices = values.into_iter().collect();
        self
    }

    /// Rejects a value in production, such as a development key.
    pub fn forbid_in_production(mut self, value: T) -> Self {
        self.forbidden_in_production = Some(value);
        self
    }

    fn describe(&self, problem: &str) -> String {
        match self.description {
            Some(description) => format!("{problem} ({description})"),
            None => problem.to_owned(),
        }
    }
}

/// Reads a variable with a custom parser.
pub fn custom<T>(parser: impl Fn(&str) -> Result<T, String> + 'static) -> Spec<T> {
    Spec {
        parser: Box::new(parser),
        description: None,
        default: None,
        dev_default: None,
        choices: Vec::new(),
        forbidden_in_production: None,
    }
}

/// Reads any type that implements `FromStr`, such as numbers and addresses.
pub fn parsed<T>() -> Spec<T>
where
    T: FromStr,
    T::Err: core::fmt::Display,
{
    custom(|raw| T::from_str(raw).map_err(|error| format!("invalid value: {error}")))
}

/// Reads exactly `N` bytes written as hexadecimal.
pub fn hex<const N: usize>() -> Spec<[u8; N]> {
    custom(|raw| {
        let invalid = || format!("must be {N} bytes of hex");
        if raw.len() != N * 2 || !raw.is_ascii() {
            return Err(invalid());
        }
        let mut bytes = [0; N];
        for (byte, pair) in bytes.iter_mut().zip(raw.as_bytes().chunks(2)) {
            let pair = core::str::from_utf8(pair).map_err(|_| invalid())?;
            *byte = u8::from_str_radix(pair, 16).map_err(|_| invalid())?;
        }
        Ok(bytes)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;

    const MODE: &str = "TEST_ENV";

    fn development(vars: &[(&str, &str)]) -> Env {
        let mut all = vec![(MODE, DEVELOPMENT)];
        all.extend_from_slice(vars);
        Env::from_vars(MODE, all)
    }

    fn production(vars: &[(&str, &str)]) -> Env {
        let mut all = vec![(MODE, PRODUCTION)];
        all.extend_from_slice(vars);
        Env::from_vars(MODE, all)
    }

    fn port() -> Spec<u16> {
        parsed::<u16>().desc("Port to listen on")
    }

    #[spec]
    fn environment() {
        given("no mode variable", || {
            let env = Env::from_vars(MODE, [("OTHER", "1")]);

            when("the mode is checked", || {
                let is_production = env.is_production();

                test("it counts as production", || {
                    assert!(is_production);
                });
            });
        });

        given("an empty mode variable", || {
            let env = Env::from_vars(MODE, [(MODE, "")]);

            when("the mode is checked", || {
                let is_production = env.is_production();

                test("it counts as production", || {
                    assert!(is_production);
                });
            });
        });

        given("an unknown mode", || {
            let env = Env::from_vars(MODE, [(MODE, "staging")]);

            when("the environment is validated", || {
                let is_production = env.is_production();
                let result = env.validate();

                test("it's rejected and treated as production", || {
                    assert!(is_production);
                    assert!(result.unwrap_err().contains("not staging"));
                });
            });
        });

        given("a development environment with no variables", || {
            let env = Env::development(MODE);

            when("the mode is checked", || {
                let is_production = env.is_production();

                test("it's development", || {
                    assert!(!is_production);
                });
            });
        });

        given("the process environment", || {
            let mode_var = "ENV_CONFIG_TEST_UNSET_VARIABLE";

            when("it's read", || {
                let env = Env::from_process(mode_var);

                test("an unset mode counts as production", || {
                    assert!(env.is_production());
                });
            });
        });
    }

    #[spec]
    fn reading() {
        given("a variable that's set", || {
            let mut env = production(&[("PORT", "9000")]);

            when("it's read", || {
                let port = env.read("PORT", port().default(1));

                test("the set value wins", || {
                    assert!(env.validate().is_ok());
                    assert_eq!(port.value(), 9000);
                });
            });
        });

        given("an unset variable with a default in production", || {
            let mut env = production(&[]);

            when("it's read", || {
                let port = env.read("PORT", port().default(1).dev_default(2));

                test("the default is used", || {
                    assert_eq!(port.value(), 1);
                });
            });
        });

        given(
            "an unset variable with a dev default in development",
            || {
                let mut env = development(&[]);

                when("it's read", || {
                    let port = env.read("PORT", port().default(1).dev_default(2));

                    test("the dev default is used", || {
                        assert_eq!(port.value(), 2);
                    });
                });
            },
        );

        given(
            "an unset variable with only a default in development",
            || {
                let mut env = development(&[]);

                when("it's read", || {
                    let port = env.read("PORT", port().default(1));

                    test("the default is used", || {
                        assert_eq!(port.value(), 1);
                    });
                });
            },
        );

        given(
            "an unset variable with only a dev default in production",
            || {
                let mut env = production(&[]);

                when("it's read and validated", || {
                    env.read("PORT", port().dev_default(2));
                    let result = env.validate();

                    test("it's reported as missing", || {
                        assert_eq!(result.unwrap_err(), "PORT: missing (Port to listen on)");
                    });
                });
            },
        );

        given("a variable with an invalid value", || {
            let mut env = production(&[("PORT", "not a port")]);

            when("it's read and validated", || {
                env.read("PORT", port());
                let result = env.validate();

                test("the problem names the variable", || {
                    assert!(result.unwrap_err().starts_with("PORT: invalid value"));
                });
            });
        });

        given("an empty variable", || {
            let mut env = production(&[("PORT", "")]);

            when("it's read", || {
                let port = env.read("PORT", port().default(1));

                test("it counts as unset", || {
                    assert_eq!(port.value(), 1);
                });
            });
        });

        given("several invalid variables", || {
            let mut env = production(&[("PORT", "bad")]);

            when("they're validated", || {
                env.read("PORT", port());
                env.read("KEY", hex::<4>());
                let result = env.validate().unwrap_err();

                test("every problem is reported", || {
                    assert!(result.contains("PORT:"));
                    assert!(result.contains("KEY: missing"));
                });
            });
        });

        given("a value outside its choices", || {
            let mut env = production(&[("PORT", "3")]);

            when("it's read and validated", || {
                env.read("PORT", port().choices([1, 2]));
                let result = env.validate();

                test("it's rejected", || {
                    assert!(result.is_err());
                });
            });
        });

        given("a value within its choices", || {
            let mut env = production(&[("PORT", "2")]);

            when("it's read and validated", || {
                env.read("PORT", port().choices([1, 2]));
                let result = env.validate();

                test("it's accepted", || {
                    assert!(result.is_ok());
                });
            });
        });

        given("a value forbidden in production", || {
            let mut env = production(&[("PORT", "1")]);

            when("it's read and validated", || {
                env.read("PORT", port().forbid_in_production(1));
                let result = env.validate();

                test("it's rejected", || {
                    assert!(result.unwrap_err().contains("can't use this value"));
                });
            });
        });

        given("the same value in development", || {
            let mut env = development(&[("PORT", "1")]);

            when("it's read and validated", || {
                env.read("PORT", port().forbid_in_production(1));
                let result = env.validate();

                test("it's accepted", || {
                    assert!(result.is_ok());
                });
            });
        });

        given("a custom parser", || {
            let mut env = production(&[("NAME", "dawn")]);
            let upper = custom(|raw| Ok(raw.to_uppercase()));

            when("it's read", || {
                let name = env.read("NAME", upper);

                test("its result is used", || {
                    assert_eq!(name.value(), "DAWN");
                });
            });
        });

        given("a variable without a description", || {
            let mut env = production(&[]);

            when("it's read and validated", || {
                env.read("PORT", parsed::<u16>());
                let result = env.validate();

                test("the message has no description", || {
                    assert_eq!(result.unwrap_err(), "PORT: missing");
                });
            });
        });
    }

    #[spec]
    fn hex_values() {
        given("valid hexadecimal", || {
            let mut env = production(&[("KEY", "01ff")]);

            when("it's read", || {
                let key = env.read("KEY", hex::<2>());

                test("the bytes are decoded", || {
                    assert_eq!(key.value(), [0x01, 0xff]);
                });
            });
        });

        given("hexadecimal of the wrong length", || {
            let mut env = production(&[("KEY", "01")]);

            when("it's read and validated", || {
                env.read("KEY", hex::<2>());
                let result = env.validate();

                test("it's rejected", || {
                    assert!(result.is_err());
                });
            });
        });

        given("text that isn't hexadecimal", || {
            let mut env = production(&[("KEY", "zzzz")]);

            when("it's read and validated", || {
                env.read("KEY", hex::<2>());
                let result = env.validate();

                test("it's rejected", || {
                    assert!(result.is_err());
                });
            });
        });
    }
}
