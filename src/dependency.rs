use crate::common::*;

#[derive(PartialEq, Debug)]
pub(crate) struct Dependency<'src>(pub(crate) Rc<Recipe<'src>>);
