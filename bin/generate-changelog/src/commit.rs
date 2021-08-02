use crate::common::*;

#[derive(Deserialize, Serialize)]
pub(crate) struct Commit {
  #[serde(with = "hex")]
  pub(crate) hash:    Vec<u8>,
  pub(crate) summary: String,
  #[serde(rename = "type")]
  pub(crate) ty:      CommitType,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) version: Option<Version>,
}

impl Commit {
  pub(crate) fn oid(&self) -> Oid {
    Oid::from_bytes(&self.hash).unwrap()
  }

  pub(crate) fn entry(&self, format: &Format) -> String {
    match format {
      Format::Markdown => {
        let (summary, pr) = if let Some(captures) = Regex::new(r"^(.*) \(#([1-9][0-9]*)\)$")
          .unwrap()
          .captures(&self.summary)
        {
          (
            captures.get(1).unwrap().as_str(),
            Some(captures[2].parse::<u64>().unwrap()),
          )
        } else {
          (self.summary.as_str(), None)
        };

        let mut entry = format!(
          "[{}](https://github.com/casey/just/commit/{})",
          summary,
          hex::encode(&self.hash)
        );

        if let Some(pr) = pr {
          entry.push_str(&format!(
            " [(#{})](https://github.com/casey/just/pull/{})",
            pr, pr
          ));
        }

        entry
      },
      Format::Text => self.summary.clone(),
    }
  }
}
