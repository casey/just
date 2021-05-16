#[cfg(windows)]
test! {
  name:     powershell,
  justfile: r#"
default:
  #!powershell
  Write-Host Hello-World
"#,
  stdout: "Hello-World\n",
}

#[cfg(windows)]
test! {
  name:     powershell_exe,
  justfile: r#"
default:
  #!powershell.exe
   Write-Host Hello-World
"#,
  stdout: "Hello-World\n",
}

#[cfg(windows)]
test! {
  name:     cmd,
  justfile: r#"
default:
  #!cmd /c
  @echo Hello-World
"#,
  stdout: "Hello-World\r\n",
}

#[cfg(windows)]
test! {
  name:     cmd_exe,
  justfile: r#"
default:
  #!cmd.exe /c
  @echo Hello-World
"#,
  stdout: "Hello-World\r\n",
}
