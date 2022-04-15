### Shell Completion Scripts

Shell completion scripts for Bash, Zsh, Fish, PowerShell, and Elvish are available in the [completions](completions) directory. Please refer to your shell’s documentation for how to install them.

The `just` binary can also generate the same completion scripts at runtime, using the `--completions` command:

````sh
$ just --completions zsh > just.zsh
````