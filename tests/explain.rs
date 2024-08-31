use super::*;

#[test]
fn explain_recipe() {
  Test::new()
    .justfile(
      "
      # List some fruits
      fruits:
        echo 'apple peach dragonfruit'
    ",
    )
    .args(["--explain", "fruits"])
    .stdout("apple peach dragonfruit\n")
    .stderr(
      "
      #### List some fruits
      echo 'apple peach dragonfruit'
    ",
    )
    .run();
}
