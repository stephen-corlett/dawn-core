//! Project automation, written in Rust so it behaves the same on every OS.
//! Run it with `cargo xtask <command>`.

mod architecture;
mod coverage;
mod test_structure;

use architecture::{CrateInfo, leaked_side_features, violations};
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::collections::BTreeSet;
use std::env;
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::thread;
use std::time::Duration;

/// Must match the line that `server` prints in its `network` module.
const SERVER_READY_LINE: &str = "DAWN_SERVER_READY";
const READY_TIMEOUT: Duration = Duration::from_secs(30);
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Crates that would mean the server is compiling graphics code.
const FORBIDDEN_SERVER_DEPS: [&str; 3] = ["wgpu", "winit", "bevy_render"];

/// The Cargo features that hold a game module's client and server code.
const SIDES: [&str; 2] = ["client", "server"];

type TaskResult = Result<(), String>;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    task: Task,
}

#[derive(Subcommand)]
enum Task {
    /// Run the server and clients together until you close them or press Ctrl-C
    Dev {
        /// How many client windows to open
        #[arg(long, default_value_t = 2)]
        clients: u64,
    },
    /// Fail if crate dependencies break docs/architecture.md
    CheckArchitecture,
    /// Build each crate alone, so none can rely on another crate's features
    CheckIsolated,
    /// Fail if either app would compile the other side's code
    CheckAppSides,
    /// Fail if a test doesn't follow the Given, When, Then format
    CheckTests,
    /// Fail if unit and integration tests cover under 90% of changed lines
    Coverage {
        /// Git ref to compare with. Defaults to where this branch left main
        #[arg(long)]
        base: Option<String>,
    },
}

fn main() -> ExitCode {
    let result = match Cli::parse().task {
        Task::Dev { clients } => dev(clients),
        Task::CheckArchitecture => check_architecture(),
        Task::CheckIsolated => check_isolated(),
        Task::CheckAppSides => check_app_sides(),
        Task::CheckTests => check_tests(),
        Task::Coverage { base } => check_coverage(base),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xtask: {error}");
            ExitCode::FAILURE
        }
    }
}

fn dev(clients: u64) -> TaskResult {
    // Building one at a time keeps Cargo from merging their feature sets,
    // which would compile graphics support into the server.
    cargo(&["build", "-p", "server"])?;
    cargo(&["build", "-p", "client", "--features", "dev"])?;

    let stop = Arc::new(AtomicBool::new(false));
    let handler_stop = stop.clone();
    ctrlc::set_handler(move || handler_stop.store(true, Ordering::SeqCst))
        .map_err(|error| format!("couldn't handle Ctrl-C: {error}"))?;

    let target_dir = workspace_root().join("target").join("debug");
    let path = runtime_path(&target_dir)?;

    let (ready_sender, ready_receiver) = channel();
    let mut server = spawn(
        "server",
        Command::new(target_dir.join("server"))
            .env("PATH", &path)
            .env("DAWN_ENV", "development"),
        Some(ready_sender),
    )?;
    if ready_receiver.recv_timeout(READY_TIMEOUT).is_err() {
        kill(&mut server);
        return Err("the server didn't become ready".into());
    }

    let mut client_processes = Vec::new();
    for id in 1..=clients {
        let label = format!("client{id}");
        let mut command = Command::new(target_dir.join("client"));
        command.env("PATH", &path).env("DAWN_ENV", "development");
        command.args(["--client-id", &id.to_string()]);
        match spawn(&label, &mut command, None) {
            Ok(child) => client_processes.push(child),
            Err(error) => {
                client_processes.iter_mut().for_each(kill);
                kill(&mut server);
                return Err(error);
            }
        }
    }

    let result = supervise(&mut server, &mut client_processes, &stop);
    client_processes.iter_mut().for_each(kill);
    kill(&mut server);
    result
}

/// Waits until Ctrl-C, the server stops, or every client window is closed.
fn supervise(server: &mut Child, clients: &mut [Child], stop: &AtomicBool) -> TaskResult {
    while !stop.load(Ordering::SeqCst) {
        if let Ok(Some(status)) = server.try_wait() {
            return Err(format!("the server stopped unexpectedly ({status})"));
        }
        let all_closed = clients
            .iter_mut()
            .all(|client| matches!(client.try_wait(), Ok(Some(_))));
        if all_closed {
            return Ok(());
        }
        thread::sleep(POLL_INTERVAL);
    }
    Ok(())
}

/// Starts a process and prefixes its output, so logs are easy to tell apart.
fn spawn(label: &str, command: &mut Command, ready: Option<Sender<()>>) -> Result<Child, String> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("couldn't start {label}: {error}"))?;
    let stdout = child.stdout.take().expect("stdout is piped");
    let stderr = child.stderr.take().expect("stderr is piped");
    prefix_output(label.to_owned(), stdout, ready);
    prefix_output(label.to_owned(), stderr, None);
    Ok(child)
}

fn prefix_output(label: String, stream: impl Read + Send + 'static, ready: Option<Sender<()>>) {
    thread::spawn(move || {
        for line in BufReader::new(stream).lines().map_while(Result::ok) {
            if line == SERVER_READY_LINE {
                if let Some(ready) = &ready {
                    let _ = ready.send(());
                }
                continue;
            }
            println!("[{label}] {line}");
        }
    });
}

fn kill(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn cargo(args: &[&str]) -> TaskResult {
    let status = Command::new("cargo")
        .args(args)
        .current_dir(workspace_root())
        .status()
        .map_err(|error| format!("couldn't run cargo: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo {} failed", args.join(" ")))
    }
}

/// `dynamic_linking` loads Bevy and Rust's standard library from shared
/// libraries. `cargo run` finds them automatically, but we launch the programs
/// directly, so their folders must be added to PATH.
fn runtime_path(target_dir: &Path) -> Result<OsString, String> {
    let output = Command::new("rustc")
        .args(["--print", "target-libdir"])
        .current_dir(workspace_root())
        .output()
        .map_err(|error| format!("couldn't run rustc: {error}"))?;
    let rust_libs = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let sysroot_bin = PathBuf::from(&rust_libs)
        .ancestors()
        .nth(4)
        .map(|sysroot| sysroot.join("bin"))
        .ok_or("couldn't find the Rust sysroot")?;

    let mut paths = vec![
        target_dir.to_path_buf(),
        target_dir.join("deps"),
        PathBuf::from(rust_libs),
        sysroot_bin,
    ];
    paths.extend(env::split_paths(&env::var_os("PATH").unwrap_or_default()));
    env::join_paths(paths).map_err(|error| error.to_string())
}

fn check_architecture() -> TaskResult {
    let crates = workspace_crates()?;
    let problems = violations(&crates);

    if problems.is_empty() {
        println!("{} crates follow the architecture rules", crates.len());
        Ok(())
    } else {
        Err(format!(
            "architecture rules broken (see docs/architecture.md):\n  {}",
            problems.join("\n  ")
        ))
    }
}

fn check_isolated() -> TaskResult {
    for info in workspace_crates()? {
        println!("checking {} alone", info.name);
        cargo(&["check", "-p", &info.name, "--all-targets"])?;
        for side in SIDES
            .into_iter()
            .filter(|side| info.features.contains(*side))
        {
            println!("checking {} with only {side}", info.name);
            cargo(&[
                "check",
                "-p",
                &info.name,
                "--all-targets",
                "--features",
                side,
            ])?;
        }
    }
    Ok(())
}

/// Reads every workspace crate's dependencies, including dev-dependencies.
fn workspace_crates() -> Result<Vec<CrateInfo>, String> {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(workspace_root())
        .output()
        .map_err(|error| format!("couldn't run cargo: {error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    let metadata: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("couldn't read cargo metadata: {error}"))?;
    let packages = metadata["packages"]
        .as_array()
        .ok_or("cargo metadata has no packages")?;

    let names: BTreeSet<&str> = packages
        .iter()
        .filter_map(|package| package["name"].as_str())
        .collect();
    let libs_dir = workspace_root().join("libs");
    let crates = packages
        .iter()
        .map(|package| {
            let deps = || package["dependencies"].as_array().into_iter().flatten();
            let dep_name = |dep: &Value| dep["name"].as_str().map(str::to_owned);
            let (workspace_deps, external_deps) = deps()
                .filter_map(dep_name)
                .partition(|dep| names.contains(dep.as_str()));
            // Cargo reports `kind` as null for normal dependencies.
            let normal_workspace_deps = deps()
                .filter(|dep| dep["kind"].is_null())
                .filter_map(dep_name)
                .filter(|dep| names.contains(dep.as_str()))
                .collect();

            let own_features = package["features"].as_object();
            let features = own_features
                .into_iter()
                .flat_map(|features| features.keys().cloned())
                .collect();
            let is_side = |feature: &str| SIDES.contains(&feature);
            // Sides can be enabled on a dependency, or forwarded by a feature
            // such as `server = ["combat/server"]`.
            let on_dependencies = deps().flat_map(|dep| {
                let name = dep["name"].as_str().unwrap_or_default();
                let requested = dep["features"].as_array().into_iter().flatten();
                requested
                    .filter_map(Value::as_str)
                    .map(move |feature| format!("{name}/{feature}"))
            });
            let forwarded = own_features
                .into_iter()
                .flat_map(|features| features.values())
                .filter_map(Value::as_array)
                .flatten()
                .filter_map(Value::as_str)
                .map(|feature| feature.replace("?/", "/"));
            let enabled_side_features = on_dependencies
                .chain(forwarded)
                .filter(|enabled| match enabled.split_once('/') {
                    Some((dep, feature)) => names.contains(dep) && is_side(feature),
                    None => false,
                })
                .collect();
            let has_default_side = package["features"]["default"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .any(is_side);

            CrateInfo {
                name: package["name"].as_str().unwrap_or_default().to_owned(),
                is_library: package["manifest_path"]
                    .as_str()
                    .is_some_and(|path| Path::new(path).starts_with(&libs_dir)),
                workspace_deps,
                normal_workspace_deps,
                external_deps,
                features,
                enabled_side_features,
                has_default_side,
            }
        })
        .collect();
    Ok(crates)
}

/// Runs every test except end-to-end with coverage, then compares it with the Git
/// diff. End-to-end tests stay out, so they can't hide a missing lower-level test.
fn check_coverage(base: Option<String>) -> TaskResult {
    let base = comparison_base(base)?;

    cargo(&["llvm-cov", "clean", "--workspace"])?;
    cargo(&[
        "llvm-cov",
        "nextest",
        "--no-report",
        "--workspace",
        "--filterset",
        "not package(e2e)",
    ])?;
    let report_path = workspace_root().join("target").join("coverage.lcov");
    let report_argument = report_path.to_string_lossy();
    cargo(&[
        "llvm-cov",
        "report",
        "--lcov",
        "--output-path",
        &report_argument,
    ])?;

    let report = std::fs::read_to_string(&report_path).map_err(|error| error.to_string())?;
    let root = workspace_root();
    let report = coverage::parse_lcov(&report, &root.to_string_lossy());
    let diff = git(&["diff", "--unified=0", "--no-color", &base])?;
    let mut changed = coverage::parse_diff(&diff);
    for file in git(&["ls-files", "--others", "--exclude-standard"])?.lines() {
        let text = std::fs::read_to_string(root.join(file)).unwrap_or_default();
        changed
            .entry(file.to_owned())
            .or_default()
            .extend(1..=text.lines().count());
    }

    for (file, lines) in &mut changed {
        let text = std::fs::read_to_string(root.join(file)).unwrap_or_default();
        if let Some(start) = coverage::test_module_start(&text) {
            lines.retain(|line| *line < start);
        }
    }

    let result = coverage::diff_coverage(&changed, &report);
    publish_to_github(&result)?;

    for (file, line, level) in result.missing_entries() {
        println!("  {file}:{line}   suggested: {level}");
    }
    let totals = format!(
        "{:.1}% of {} changed lines since {base} are covered, and the minimum is {}%",
        result.percent(),
        result.executable_lines,
        coverage::MIN_DIFF_COVERAGE_PERCENT,
    );
    if result.passes() {
        println!("{totals}");
        Ok(())
    } else {
        Err(totals)
    }
}

/// Adds a job summary, inline annotations, and comment text in GitHub Actions.
fn publish_to_github(result: &coverage::DiffCoverage) -> TaskResult {
    if let Some(path) = env::var_os("COVERAGE_COMMENT_PATH") {
        std::fs::write(path, result.comment_markdown())
            .map_err(|error| format!("couldn't write the coverage comment: {error}"))?;
    }
    if env::var_os("GITHUB_ACTIONS").is_some() {
        for annotation in result.annotations() {
            println!("{annotation}");
        }
    }
    if let Some(path) = env::var_os("GITHUB_STEP_SUMMARY") {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|error| format!("couldn't open the job summary: {error}"))?;
        file.write_all(result.summary_markdown().as_bytes())
            .map_err(|error| format!("couldn't write the job summary: {error}"))?;
    }
    Ok(())
}

fn comparison_base(base: Option<String>) -> Result<String, String> {
    if git(&["rev-parse", "--verify", "--quiet", "HEAD"]).is_err() {
        return Err("coverage compares with Git history, so it needs a first commit".into());
    }
    match base {
        Some(base) => Ok(base),
        // Local main can fall behind, so prefer the copy from GitHub.
        None => Ok(git(&["merge-base", "origin/main", "HEAD"])
            .or_else(|_| git(&["merge-base", "main", "HEAD"]))?
            .trim()
            .to_owned()),
    }
}

fn git(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(workspace_root())
        .output()
        .map_err(|error| format!("couldn't run git: {error}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn check_tests() -> TaskResult {
    let mut files = vec![workspace_root().join("Cargo.toml")];
    for folder in ["crates", "libs", "xtask"] {
        source_files(&workspace_root().join(folder), &mut files)?;
    }

    let mut problems = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file).map_err(|error| error.to_string())?;
        let found = if file.ends_with("Cargo.toml") {
            test_structure::manifest_problems(&text)
        } else {
            test_structure::problems(&text)
        };
        for problem in found {
            problems.push(format!("{}: {problem}", file.display()));
        }
    }

    if problems.is_empty() {
        println!("{} source files follow the test format", files.len());
        Ok(())
    } else {
        Err(format!(
            "tests break the format in docs/architecture.md:\n  {}",
            problems.join("\n  ")
        ))
    }
}

/// Collects Rust source files and crate manifests.
fn source_files(dir: &Path, files: &mut Vec<PathBuf>) -> TaskResult {
    for entry in std::fs::read_dir(dir).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.is_dir() {
            source_files(&path, files)?;
        } else if path.ends_with("Cargo.toml")
            || path.extension().is_some_and(|extension| extension == "rs")
        {
            files.push(path);
        }
    }
    Ok(())
}

/// Checks what each app would really compile when built on its own. Cargo merges
/// features across a workspace-wide build, so ship each app with `-p`.
fn check_app_sides() -> TaskResult {
    let crates = workspace_crates()?;
    let server_tree = dependency_tree("server")?;
    let client_tree = dependency_tree("client")?;

    let mut problems: Vec<String> = FORBIDDEN_SERVER_DEPS
        .into_iter()
        .filter(|name| {
            server_tree
                .lines()
                .any(|line| line.starts_with(&format!("{name} ")))
        })
        .map(|name| format!("server depends on {name}"))
        .collect();
    for leak in leaked_side_features(&server_tree, "client", &crates) {
        problems.push(format!("server compiles {leak}"));
    }
    for leak in leaked_side_features(&client_tree, "server", &crates) {
        problems.push(format!("client compiles {leak}"));
    }

    if problems.is_empty() {
        println!("the server and client apps only compile their own side");
        Ok(())
    } else {
        Err(problems.join("\n"))
    }
}

fn dependency_tree(package: &str) -> Result<String, String> {
    let output = Command::new("cargo")
        .args([
            "tree",
            "-p",
            package,
            "--edges",
            "normal,features",
            "--prefix",
            "none",
        ])
        .current_dir(workspace_root())
        .output()
        .map_err(|error| format!("couldn't run cargo: {error}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask is inside the workspace")
        .to_path_buf()
}
