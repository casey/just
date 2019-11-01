#[derive(PartialEq)]
pub(crate) enum Subcommand<'a> {
  Dump,
  Edit,
  Evaluate,
  List,
  Run,
  Show { name: &'a str },
  Summary,
}
