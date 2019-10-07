#[derive(PartialEq)]
pub(crate) enum Subcommand<'a> {
  Edit,
  Summary,
  Dump,
  List,
  Show { name: &'a str },
  Run,
}
