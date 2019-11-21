use crate::common::*;

#[derive(PartialEq, Debug)]
pub(crate) struct Dependency<'a>(pub(crate) Rc<Recipe<'a>>);
