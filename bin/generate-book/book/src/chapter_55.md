### Shell Alias

For lightning-fast command running, put `alias j=just` in your shell’s configuration file.

In `bash`, the aliased command may not keep the shell completion functionality described in the next section. Add the following line to your `.bashrc` to use the same completion function as `just` for your aliased command:

````sh
complete -F _just -o bashdefault -o default j
````