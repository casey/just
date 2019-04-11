use crate::common::*;

pub fn load_dotenv() -> RunResult<'static, Map<String, String>> {
  match dotenv::dotenv_iter() {
    Ok(iter) => {
      let result: dotenv::Result<Map<String, String>> = iter.collect();
      result.map_err(|dotenv_error| RuntimeError::Dotenv { dotenv_error })
    }
    Err(dotenv_error) => {
      if dotenv_error.not_found() {
        Ok(Map::new())
      } else {
        Err(RuntimeError::Dotenv { dotenv_error })
      }
    }
  }
}
