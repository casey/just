use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Request {
  EnvironmentVariable(String),
  #[cfg(not(windows))]
  Signal,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Response {
  EnvironmentVariable(Option<OsString>),
  #[cfg(not(windows))]
  Signal(String),
}
