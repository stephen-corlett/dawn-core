//! Compares the installed Bevy and the crates built on it with what's published,
//! so the upgrade skill starts from facts rather than from a release announcement.

use serde_json::Value;
use std::path::Path;
use std::process::Command;

/// Third-party crates that must support a Bevy version before we can move to it.
/// Add a crate here when the workspace starts depending on one that's built on Bevy.
const BUILT_ON_BEVY: [&str; 1] = ["lightyear"];

/// A crate built on Bevy may depend on the whole engine or only on these parts of it.
const BEVY_CRATES: [&str; 3] = ["bevy", "bevy_app", "bevy_ecs"];

const CRATES_IO: &str = "https://crates.io/api/v1/crates";
const GUIDES: &str =
    "https://raw.githubusercontent.com/bevyengine/bevy-website/main/content/learn/migration-guides";

/// crates.io rejects requests that don't say who is calling.
const USER_AGENT: &str = "dawn-core-xtask (https://github.com/stephen-corlett/dawn-core)";

struct Release {
    version: String,
    rust: String,
}

struct Plugin {
    name: &'static str,
    installed: String,
    release: Release,
    bevy_minor: u64,
}

pub fn status(root: &Path) -> Result<(), String> {
    let lockfile = std::fs::read_to_string(root.join("Cargo.lock"))
        .map_err(|error| format!("couldn't read Cargo.lock: {error}"))?;

    let installed = locked_version(&lockfile, "bevy")?;
    let bevy = latest_release("bevy")?;
    println!(
        "bevy: {installed} installed, {} published, needs Rust {}",
        bevy.version, bevy.rust
    );

    let mut plugins = Vec::new();
    for name in BUILT_ON_BEVY {
        let release = latest_release(name)?;
        let plugin = Plugin {
            name,
            installed: locked_version(&lockfile, name)?,
            bevy_minor: bevy_minor_required(name, &release.version)?,
            release,
        };
        println!(
            "{name}: {} installed, {} published, needs Bevy 0.{} and Rust {}",
            plugin.installed, plugin.release.version, plugin.bevy_minor, plugin.release.rust
        );
        plugins.push(plugin);
    }
    println!();

    let installed_minor = minor(&installed)?;
    let latest_minor = minor(&bevy.version)?;
    let supported = plugins
        .iter()
        .map(|plugin| plugin.bevy_minor)
        .fold(latest_minor, u64::min);
    let lagging = plugins
        .iter()
        .filter(|plugin| plugin.bevy_minor < latest_minor)
        .map(|plugin| plugin.name)
        .collect::<Vec<_>>()
        .join(", ");

    if supported > installed_minor {
        println!("Upgrade available: Bevy 0.{installed_minor} to 0.{supported}");
        println!("Migration guides, in order:");
        for from in installed_minor..supported {
            println!("  {GUIDES}/0.{from}-to-0.{}.md", from + 1);
        }
        if supported < latest_minor {
            println!("Bevy 0.{latest_minor} is out, but {lagging} doesn't support it yet.");
        }
    } else if latest_minor > installed_minor {
        println!("Waiting: Bevy 0.{latest_minor} is out, but {lagging} doesn't support it yet.");
    } else if installed != bevy.version
        || plugins
            .iter()
            .any(|plugin| plugin.installed != plugin.release.version)
    {
        println!("Update available within Bevy 0.{installed_minor}.");
    } else {
        println!("Up to date.");
    }
    Ok(())
}

/// Finds a package's version in Cargo.lock, which lists `version` right after `name`.
fn locked_version(lockfile: &str, name: &str) -> Result<String, String> {
    let name_line = format!("name = \"{name}\"");
    let mut lines = lockfile.lines();
    lines
        .find(|line| *line == name_line)
        .and_then(|_| lines.next())
        .and_then(|line| line.strip_prefix("version = \""))
        .and_then(|version| version.strip_suffix('"'))
        .map(str::to_owned)
        .ok_or_else(|| format!("{name} isn't in Cargo.lock"))
}

/// Reads the `20` from a version such as `0.20.1`, or a requirement such as `^0.20`.
fn minor(version: &str) -> Result<u64, String> {
    let digits = version.trim_start_matches(|character: char| !character.is_ascii_digit());
    match digits.split('.').collect::<Vec<_>>()[..] {
        ["0", minor, ..] => minor.parse().ok(),
        _ => None,
    }
    .ok_or_else(|| format!("expected a 0.x version, but found {version}"))
}

/// Skips prereleases, so a release candidate is never offered as an upgrade.
fn latest_release(name: &str) -> Result<Release, String> {
    let summary = crates_io(name)?;
    let version = summary["crate"]["max_stable_version"]
        .as_str()
        .ok_or_else(|| format!("crates.io has no stable release of {name}"))?;
    let details = crates_io(&format!("{name}/{version}"))?;
    Ok(Release {
        version: version.to_owned(),
        rust: details["version"]["rust_version"]
            .as_str()
            .unwrap_or("unknown")
            .to_owned(),
    })
}

/// Reads a release's dependencies. This is what Cargo will enforce, so it's more
/// reliable than a compatibility table in a README.
fn bevy_minor_required(name: &str, version: &str) -> Result<u64, String> {
    let dependencies = crates_io(&format!("{name}/{version}/dependencies"))?;
    let requirement = dependencies["dependencies"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|dependency| dependency["kind"] == "normal")
        .find(|dependency| {
            let id = dependency["crate_id"].as_str().unwrap_or_default();
            BEVY_CRATES.contains(&id)
        })
        .and_then(|dependency| dependency["req"].as_str())
        .ok_or_else(|| format!("{name} {version} doesn't depend on Bevy"))?;
    minor(requirement)
}

fn crates_io(path: &str) -> Result<Value, String> {
    let output = Command::new("curl")
        .args(["--silent", "--show-error", "--fail", "--user-agent"])
        .arg(USER_AGENT)
        .arg(format!("{CRATES_IO}/{path}"))
        .output()
        .map_err(|error| format!("couldn't run curl: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "couldn't read {path} from crates.io: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("couldn't read the crates.io response: {error}"))
}
