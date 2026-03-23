use semver::Version;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::process::{Command, Stdio};
use tabled::settings::style::{BorderSpanCorrection, Style};
use tabled::{Table, Tabled};

#[derive(Deserialize)]
struct Workflow {
  jobs: BTreeMap<String, Job>,
}

#[derive(Deserialize)]
struct Job {
  #[serde(default)]
  steps: Vec<Step>,
}

#[derive(Deserialize)]
struct Step {
  uses: Option<String>,
}

fn parse_version(s: &str) -> Option<Version> {
  let s = s.strip_prefix('v').unwrap_or(s);
  if s.contains('.') {
    Version::parse(s).ok()
  } else {
    Version::parse(&format!("{s}.0.0")).ok()
  }
}

fn latest_version(repo: &str) -> Option<String> {
  let output = Command::new("gh")
    .args([
      "api",
      &format!("repos/{repo}/releases/latest"),
      "--jq",
      ".tag_name",
    ])
    .stderr(Stdio::null())
    .output()
    .ok()?;

  if output.status.success() {
    let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !tag.is_empty() {
      return Some(tag);
    }
  }

  let output = Command::new("gh")
    .args(["api", &format!("repos/{repo}/tags"), "--jq", ".[0].name"])
    .stderr(Stdio::null())
    .output()
    .ok()?;

  if output.status.success() {
    let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !tag.is_empty() {
      return Some(tag);
    }
  }

  None
}

fn main() {
  let mut seen = BTreeMap::<String, String>::new();

  let entries: Vec<_> = fs::read_dir(".github/workflows")
    .expect("could not read .github/workflows")
    .filter_map(|e| e.ok())
    .filter(|e| {
      let name = e.file_name();
      let name = name.to_string_lossy();
      name.ends_with(".yaml") || name.ends_with(".yml")
    })
    .collect();

  for entry in &entries {
    let path = entry.path();
    let content = fs::read_to_string(&path).expect("could not read workflow file");
    let workflow: Workflow = serde_yaml::from_str(&content).expect("could not parse workflow file");

    for job in workflow.jobs.values() {
      for step in &job.steps {
        let Some(uses) = &step.uses else { continue };

        if uses.starts_with("./") || uses.starts_with("docker://") {
          continue;
        }

        let Some((repo, version)) = uses.split_once('@') else {
          continue;
        };

        seen
          .entry(repo.to_string())
          .or_insert_with(|| version.to_string());
      }
    }
  }

  #[derive(Tabled)]
  struct Row {
    action: String,
    version: String,
    latest: String,
    status: String,
  }

  let mut rows = Vec::new();
  let mut ok = true;

  for (repo, version) in &seen {
    match latest_version(repo) {
      Some(latest) => {
        let current = parse_version(version);
        let latest_ver = parse_version(&latest);

        let matches = match (current, latest_ver) {
          (Some(current), Some(latest_ver)) => current.major == latest_ver.major,
          _ => version == &latest,
        };

        let status = if matches { "ok" } else { "mismatch" };

        if !matches {
          ok = false;
        }

        rows.push(Row {
          action: repo.clone(),
          version: version.clone(),
          latest: latest.clone(),
          status: status.to_string(),
        });
      }
      None => {
        rows.push(Row {
          action: repo.clone(),
          version: version.clone(),
          latest: "?".to_string(),
          status: "warning".to_string(),
        });
      }
    }
  }

  let mut table = Table::new(&rows);
  table.with(Style::modern()).with(BorderSpanCorrection);
  println!("{table}");

  if !ok {
    std::process::exit(1);
  }
}
