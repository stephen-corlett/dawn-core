//! Enforces the crate rules in docs/architecture.md.
//!
//! Cargo already rejects cycles through normal dependencies, but it allows
//! them through dev-dependencies, so this check covers both.

use std::collections::{BTreeMap, BTreeSet};

/// `foundation` may only use these external crates.
const FOUNDATION_EXTERNAL_DEPS: [&str; 3] = ["bevy", "lightyear", "serde"];

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
        Foundation | Tooling => false,
        Library => to == Library,
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
