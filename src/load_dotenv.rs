use crate::common::*;

pub(crate) fn load_dotenv() -> RunResult<'static, BTreeMap<String, String>> {
  // `dotenv::dotenv_iter` should eventually be un-deprecated, see:
  // https://github.com/dotenv-rs/dotenv/issues/13
  #![allow(deprecated)]
  match dotenv::dotenv_iter() {
    Ok(iter) => {
      let result: dotenv::Result<BTreeMap<String, String>> = iter.collect();
      result.map_err(|dotenv_error| RuntimeError::Dotenv { dotenv_error })
    }
    Err(dotenv_error) => {
      if dotenv_error.not_found() {
        Ok(BTreeMap::new())
      } else {
        Err(RuntimeError::Dotenv { dotenv_error })
      }
    }
  }
}
