# Non-Project Specific Justfile

If you want some commands to be available everywhere, put them in `~/.justfile` and add the following to your shell's initialization file:

```sh
alias .j='just --justfile ~/.justfile --working-directory ~'
```

Or, if you'd rather they run in the current directory:

```sh
alias .j='just --justfile ~/.justfile --working-directory .'
```

I'm pretty sure that nobody actually uses this feature, but it's there.

¯\\_(ツ)_/¯
