extern crate self_update;

use crate::common::*;

pub(crate) fn update_just() -> Result<(), Box<dyn ::std::error::Error>> {
  let _status = self_update::backends::github::Update::configure()
    .repo_owner("casey")
    .repo_name(env!("CARGO_PKG_NAME"))
    .bin_name(env!("CARGO_PKG_NAME"))
    .target("x86_64-unknown-linux-musl") // TODO: generalize. Either a map lookup or default to x86_64-unknown-linux-gnu archive name. 
    .show_download_progress(true)
    .current_version(env!("CARGO_PKG_VERSION"))
    .build()?
    .update()?;
  println!("✅ Done.");
  Ok(())
}
