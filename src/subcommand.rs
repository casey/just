#[derive(PartialEq, Clone, Debug)]
pub(crate) enum Subcommand {
  Dump,
  Edit,
  Evaluate,
  Run,
  List,
  Show { name: String },
  Summary,
}
