//! Enforces the crate rules in docs/architecture.md.
//!
//! Cargo already rejects cycles through normal dependencies, but it allows
//! them through dev-dependencies, so this check covers both.

use std::collections::{BTreeMap, BTreeSet};

/// `foundation` may only use these external crates.
const FOUNDATION_EXTERNAL_DEPS: [&str; 3] = ["bevy", "lightyear", "serde"];

/// Keeps `foundation` small, because every line of it can trigger a full rebuild.
pub const FOUNDATION_MAX_LINES: usize = 300;

/// A workspace crate and the dependencies it declares, of every kind.
#[derive(Clone, Debug)]
pub struct CrateInfo {
    pub name: String,
    /// Whether this crate lives in `libs/`.
    pub is_library: bool,
    pub workspace_deps: BTreeSet<String>,
    /// The subset of `workspace_deps` that ships in non-test builds.
    pub normal_workspace_deps: BTreeSet<String>,
    pub external_deps: BTreeSet<String>,
    /// The features this crate defines itself.
    pub features: BTreeSet<String>,
    /// Client or server features this crate turns on in other workspace crates,
    /// written as `crate/feature`.
    pub enabled_side_features: BTreeSet<String>,
    /// Whether a side feature is on by default, which would leak it everywhere.
    pub has_default_side: bool,
}

/// Libraries for tests and experiments, which must never ship in the game.
const DEV_ONLY_LIBRARIES: [&str; 2] = ["bdd", "sandbox_kit"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Layer {
    Library,
    Foundation,
    Protocol,
    Module,
    AppServer,
    AppClient,
    EndToEnd,
    TestSupport,
    Tooling,
}

fn layer_of(name: &str) -> Layer {
    match name {
        "foundation" => Layer::Foundation,
        "protocol" => Layer::Protocol,
        "server" => Layer::AppServer,
        "client" => Layer::AppClient,
        "e2e" => Layer::EndToEnd,
        "test_support" => Layer::TestSupport,
        "xtask" => Layer::Tooling,
        _ => Layer::Module,
    }
}

fn may_depend_on(from: Layer, to: Layer) -> bool {
    use Layer::*;
    match from {
        Foundation => false,
        Library | Tooling => to == Library,
        Protocol | Module => matches!(to, Library | Foundation | Module),
        AppServer | AppClient => {
            matches!(to, Library | Foundation | Protocol | Module | TestSupport)
        }
        TestSupport => matches!(to, Library | Foundation | Protocol | Module),
        EndToEnd => to != Tooling,
    }
}

/// Only apps choose a side, because a module turning on another module's side
/// would pull that code into builds that never asked for it.
fn may_enable_side(from: Layer, side: &str) -> bool {
    match from {
        Layer::AppServer => side == "server",
        Layer::AppClient => side == "client",
        Layer::EndToEnd => true,
        _ => false,
    }
}

/// Finds workspace crates in a `cargo tree --edges normal,features` listing that
/// were built with `side` turned on.
pub fn leaked_side_features(tree: &str, side: &str, crates: &[CrateInfo]) -> BTreeSet<String> {
    let label = format!("feature \"{side}\"");
    tree.lines()
        .filter(|line| match line.split_once(' ') {
            Some((name, rest)) => rest == label && crates.iter().any(|info| info.name == name),
            None => false,
        })
        .map(str::to_owned)
        .collect()
}

/// Returns every rule broken by this set of crates.
pub fn violations(crates: &[CrateInfo]) -> Vec<String> {
    let libraries: BTreeSet<&str> = crates
        .iter()
        .filter(|info| info.is_library)
        .map(|info| info.name.as_str())
        .collect();
    let layer = |name: &str| {
        if libraries.contains(name) {
            Layer::Library
        } else {
            layer_of(name)
        }
    };

    let mut problems = Vec::new();
    for info in crates {
        let from = layer(&info.name);
        if from == Layer::Module
            && (info.name.ends_with("_server") || info.name.ends_with("_client"))
        {
            problems.push(format!(
                "{} must be a `server` or `client` feature of its game module",
                info.name
            ));
        }
        if info.has_default_side {
            problems.push(format!(
                "{} must not turn on `client` or `server` by default",
                info.name
            ));
        }
        for enabled in &info.enabled_side_features {
            let side = enabled.rsplit('/').next().unwrap_or_default();
            if !may_enable_side(from, side) {
                problems.push(format!("{} must not enable {enabled}", info.name));
            }
        }
        for dep in &info.workspace_deps {
            if !may_depend_on(from, layer(dep)) {
                problems.push(format!(
                    "{} ({from:?}) must not depend on {dep} ({:?})",
                    info.name,
                    layer(dep),
                ));
            }
        }
        for dep in &info.normal_workspace_deps {
            if layer(dep) == Layer::TestSupport || DEV_ONLY_LIBRARIES.contains(&dep.as_str()) {
                problems.push(format!(
                    "{} must list {dep} under [dev-dependencies]",
                    info.name
                ));
            }
        }
        if from == Layer::Foundation {
            for dep in &info.external_deps {
                if !FOUNDATION_EXTERNAL_DEPS.contains(&dep.as_str()) {
                    problems.push(format!(
                        "foundation must not depend on external crate {dep}"
                    ));
                }
            }
        }
    }
    problems.extend(cycles(crates));
    problems
}

fn cycles(crates: &[CrateInfo]) -> Vec<String> {
    let graph: BTreeMap<&str, &BTreeSet<String>> = crates
        .iter()
        .map(|info| (info.name.as_str(), &info.workspace_deps))
        .collect();
    let mut finished = BTreeSet::new();
    let mut found = Vec::new();
    for start in graph.keys() {
        visit(start, &graph, &mut Vec::new(), &mut finished, &mut found);
    }
    found
}

fn visit<'a>(
    name: &'a str,
    graph: &BTreeMap<&'a str, &'a BTreeSet<String>>,
    path: &mut Vec<&'a str>,
    finished: &mut BTreeSet<&'a str>,
    found: &mut Vec<String>,
) {
    if let Some(start) = path.iter().position(|step| *step == name) {
        found.push(format!(
            "dependency cycle: {} -> {name}",
            path[start..].join(" -> ")
        ));
        return;
    }
    if finished.contains(name) {
        return;
    }
    path.push(name);
    if let Some(deps) = graph.get(name) {
        for dep in deps.iter() {
            visit(dep, graph, path, finished, found);
        }
    }
    path.pop();
    finished.insert(name);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdd::spec;

    fn krate(name: &str, deps: &[&str]) -> CrateInfo {
        CrateInfo {
            name: name.into(),
            is_library: false,
            workspace_deps: deps.iter().map(|dep| dep.to_string()).collect(),
            normal_workspace_deps: deps.iter().map(|dep| dep.to_string()).collect(),
            external_deps: BTreeSet::new(),
            features: BTreeSet::new(),
            enabled_side_features: BTreeSet::new(),
            has_default_side: false,
        }
    }

    fn library(name: &str, deps: &[&str]) -> CrateInfo {
        CrateInfo {
            is_library: true,
            ..krate(name, deps)
        }
    }

    fn enabling(info: CrateInfo, feature: &str) -> CrateInfo {
        CrateInfo {
            enabled_side_features: BTreeSet::from([feature.to_owned()]),
            ..info
        }
    }

    #[spec]
    fn architecture_rules() {
        given("a layout where every crate follows the rules", || {
            let crates = [
                krate("foundation", &[]),
                krate("combat", &["foundation", "movement"]),
                krate("movement", &["foundation"]),
                krate("protocol", &["foundation", "combat"]),
                enabling(
                    krate("server", &["foundation", "protocol", "combat"]),
                    "combat/server",
                ),
                enabling(krate("client", &["protocol", "combat"]), "combat/client"),
                krate("e2e", &["server", "client"]),
            ];

            when("the rules are checked", || {
                let problems = violations(&crates);

                test("nothing is reported", || {
                    assert_eq!(problems, Vec::<String>::new());
                });
            });
        });

        given(
            "a client app that enables a module's server feature",
            || {
                let crates = [
                    enabling(krate("client", &["loot"]), "loot/server"),
                    krate("loot", &[]),
                ];

                when("the rules are checked", || {
                    let problems = violations(&crates);

                    test("one problem is reported", || {
                        assert_eq!(problems.len(), 1);
                    });
                });
            },
        );

        given(
            "a server app that enables a module's client feature",
            || {
                let crates = [
                    enabling(krate("server", &["loot"]), "loot/client"),
                    krate("loot", &[]),
                ];

                when("the rules are checked", || {
                    let problems = violations(&crates);

                    test("one problem is reported", || {
                        assert_eq!(problems.len(), 1);
                    });
                });
            },
        );

        given(
            "a module that enables another module's server feature",
            || {
                let crates = [
                    enabling(krate("combat", &["loot"]), "loot/server"),
                    krate("loot", &[]),
                ];

                when("the rules are checked", || {
                    let problems = violations(&crates);

                    test("one problem is reported", || {
                        assert_eq!(problems.len(), 1);
                    });
                });
            },
        );

        given("a module with a side turned on by default", || {
            let module = CrateInfo {
                has_default_side: true,
                ..krate("loot", &[])
            };

            when("the rules are checked", || {
                let problems = violations(&[module]);

                test("one problem is reported", || {
                    assert_eq!(problems.len(), 1);
                });
            });
        });

        given("a crate named like a module's server side", || {
            let crates = [krate("loot_server", &[])];

            when("the rules are checked", || {
                let problems = violations(&crates);

                test("one problem is reported", || {
                    assert_eq!(problems.len(), 1);
                });
            });
        });

        given("a crate named like a module's client side", || {
            let crates = [krate("loot_client", &[])];

            when("the rules are checked", || {
                let problems = violations(&crates);

                test("one problem is reported", || {
                    assert_eq!(problems.len(), 1);
                });
            });
        });

        given(
            "a tree where a workspace crate has the server feature",
            || {
                let tree =
                    "movement v0.1.0\nmovement feature \"server\"\nbevy feature \"server\"\n";
                let crates = [krate("movement", &[])];

                when("the tree is searched for the server side", || {
                    let leaks = leaked_side_features(tree, "server", &crates);

                    test("only the workspace crate is reported", || {
                        assert_eq!(leaks.len(), 1);
                    });
                });
            },
        );

        given("a tree where no crate has the client feature", || {
            let tree = "movement v0.1.0\nmovement feature \"server\"\n";
            let crates = [krate("movement", &[])];

            when("the tree is searched for the client side", || {
                let leaks = leaked_side_features(tree, "client", &crates);

                test("nothing is reported", || {
                    assert!(leaks.is_empty());
                });
            });
        });

        given("a module using a library that uses another library", || {
            let crates = [
                krate("movement", &["pathfinding"]),
                library("pathfinding", &["geometry"]),
                library("geometry", &[]),
            ];

            when("the rules are checked", || {
                let problems = violations(&crates);

                test("nothing is reported", || {
                    assert_eq!(problems, Vec::<String>::new());
                });
            });
        });

        given("a library that depends on a game module", || {
            let crates = [
                library("pathfinding", &["movement"]),
                krate("movement", &[]),
            ];

            when("the rules are checked", || {
                let problems = violations(&crates);

                test("one problem is reported", || {
                    assert_eq!(problems.len(), 1);
                });
            });
        });

        given(
            "a module that lists sandbox_kit only as a dev-dependency",
            || {
                let mut movement = krate("movement", &["sandbox_kit"]);
                movement.normal_workspace_deps.clear();
                let crates = [movement, library("sandbox_kit", &[])];

                when("the rules are checked", || {
                    let problems = violations(&crates);

                    test("nothing is reported", || {
                        assert_eq!(problems, Vec::<String>::new());
                    });
                });
            },
        );

        given("a module with sandbox_kit as a normal dependency", || {
            let crates = [
                krate("movement", &["sandbox_kit"]),
                library("sandbox_kit", &[]),
            ];

            when("the rules are checked", || {
                let problems = violations(&crates);

                test("one problem is reported", || {
                    assert_eq!(problems.len(), 1);
                });
            });
        });

        given(
            "an app that lists test_support only as a dev-dependency",
            || {
                let mut server = krate("server", &["test_support"]);
                server.normal_workspace_deps.clear();
                let crates = [server, krate("test_support", &[])];

                when("the rules are checked", || {
                    let problems = violations(&crates);

                    test("nothing is reported", || {
                        assert_eq!(problems, Vec::<String>::new());
                    });
                });
            },
        );

        given("an app with test_support as a normal dependency", || {
            let crates = [
                krate("server", &["test_support"]),
                krate("test_support", &[]),
            ];

            when("the rules are checked", || {
                let problems = violations(&crates);

                test("one problem is reported", || {
                    assert_eq!(problems.len(), 1);
                });
            });
        });

        given("test_support depending on the real server", || {
            let crates = [krate("test_support", &["server"]), krate("server", &[])];

            when("the rules are checked", || {
                let problems = violations(&crates);

                test("one problem is reported", || {
                    assert_eq!(problems.len(), 1);
                });
            });
        });

        given("foundation depending on a game module", || {
            let crates = [krate("foundation", &["movement"]), krate("movement", &[])];

            when("the rules are checked", || {
                let problems = violations(&crates);

                test("one problem is reported", || {
                    assert_eq!(problems.len(), 1);
                });
            });
        });

        given(
            "foundation with an external crate outside its allowed list",
            || {
                let mut foundation = krate("foundation", &[]);
                foundation.external_deps.insert("rand".into());

                when("the rules are checked", || {
                    let problems = violations(&[foundation]);

                    test("one problem is reported", || {
                        assert_eq!(problems.len(), 1);
                    });
                });
            },
        );

        given("a game module depending on the server app", || {
            let crates = [krate("movement", &["server"]), krate("server", &[])];

            when("the rules are checked", || {
                let problems = violations(&crates);

                test("one problem is reported", || {
                    assert_eq!(problems.len(), 1);
                });
            });
        });

        given("two modules that depend on each other", || {
            let crates = [krate("combat", &["loot"]), krate("loot", &["combat"])];

            when("the rules are checked", || {
                let problems = violations(&crates);

                test("a single dependency cycle is reported", || {
                    assert_eq!(problems.len(), 1);
                    assert!(problems[0].starts_with("dependency cycle"));
                });
            });
        });
    }
}
