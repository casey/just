use crate::common::*;

pub(crate) fn load_dotenv() -> RunResult<'static, BTreeMap<String, String>> {
  // `dotenv::dotenv_iter` should eventually be un-deprecated, see:
  // https://github.com/dotenv-rs/dotenv/issues/13
  #![allow(deprecated)]
  match dotenv::dotenv_iter() {
    Ok(iter) => {
      let mut dotenv = BTreeMap::new();
      for result in iter {
        let (key, value) = result.map_err(|dotenv_error| RuntimeError::Dotenv { dotenv_error })?;
        if env::var_os(&key).is_none() {
          dotenv.insert(key, value);
        }
      }
      Ok(dotenv)
    },
    Err(dotenv_error) =>
      if dotenv_error.not_found() {
        Ok(BTreeMap::new())
      } else {
        Err(RuntimeError::Dotenv { dotenv_error })
      },
  }
}
