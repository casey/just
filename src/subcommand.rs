use crate::common::*;

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum Subcommand {
  Dump,
  Edit,
  Evaluate {
    overrides: BTreeMap<String, String>,
  },
  Run {
    overrides: BTreeMap<String, String>,
    arguments: Vec<String>,
  },
  List,
  Show {
    name: String,
  },
  Summary,
}
