use {
  crate::error::Error,
  semver::Version,
  serde::Deserialize,
  snafu::{ErrorCompat, ResultExt, Snafu},
  std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    process::{Command, ExitCode, ExitStatus},
    str::Utf8Error,
  },
  tabled::{
    settings::style::{BorderSpanCorrection, Style},
    {Table, Tabled},
  },
};

mod error;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Deserialize)]
struct Workflow {
  jobs: BTreeMap<String, Job>,
}

#[derive(Deserialize)]
struct Job {
  steps: Vec<Step>,
}

#[derive(Deserialize)]
struct Step {
  uses: Option<String>,
}

#[derive(Tabled)]
struct Row {
  action: String,
  version: Version,
  latest: Version,
  status: &'static str,
}

fn parse_version(version: &str) -> Result<Version> {
  let version = version.strip_prefix('v').unwrap_or(version);
  let version = if version.contains('.') {
    version.to_string()
  } else {
    format!("{version}.0.0")
  };
  Version::parse(&version).context(error::ParseVersion { version })
}

fn latest_version(repo: &str) -> Result<Version> {
  let output = Command::new("gh")
    .args([
      "api",
      &format!("repos/{repo}/releases/latest"),
      "--jq",
      ".tag_name",
    ])
    .output()
    .context(error::GhInvoke { repo })?;

  if !output.status.success() {
    return Err(
      error::GhStatus {
        status: output.status,
        repo,
      }
      .build(),
    );
  }

  parse_version(
    str::from_utf8(&output.stdout)
      .context(error::GhOutput)?
      .trim(),
  )
}

const WORKFLOWS: &str = ".github/workflows";

fn run() -> Result<()> {
  let mut actions = BTreeSet::new();

  for entry in fs::read_dir(WORKFLOWS).context(error::Io { path: WORKFLOWS })? {
    let entry = entry.context(error::Io { path: WORKFLOWS })?;

    let path = entry.path();

    let Some(extension) = path.extension() else {
      continue;
    };

    if !(extension == "yaml" || extension == "yml") {
      continue;
    }

    let yaml = fs::read_to_string(&path).context(error::Io { path: &path })?;

    let workflow =
      serde_yaml::from_str::<Workflow>(&yaml).context(error::ParseWorkflow { path: &path })?;

    for job in workflow.jobs.values() {
      for step in &job.steps {
        let Some(uses) = &step.uses else { continue };

        let Some((action, version)) = uses.split_once('@') else {
          return Err(error::UsesParse { uses }.build());
        };

        actions.insert((action.to_string(), parse_version(version)?));
      }
    }
  }

  let mut rows = Vec::new();

  for (action, version) in actions {
    let latest = latest_version(&action)?;

    let status = if version.major == latest.major {
      "ok"
    } else {
      "mismatch"
    };

    rows.push(Row {
      action,
      latest,
      status,
      version,
    });
  }

  println!(
    "{}",
    Table::new(&rows)
      .with(Style::modern())
      .with(BorderSpanCorrection),
  );

  if rows.iter().any(|row| row.status == "mismatch") {
    return Err(Error::Mismatch);
  }

  Ok(())
}

fn main() -> ExitCode {
  match run() {
    Ok(()) => ExitCode::SUCCESS,
    Err(err) => {
      eprintln!("error: {err}");

      for (i, err) in err.iter_chain().skip(1).enumerate() {
        if i == 0 {
          eprintln!();
          eprintln!("because:");
        }

        eprintln!("- {err}");
      }

      ExitCode::FAILURE
    }
  }
}
