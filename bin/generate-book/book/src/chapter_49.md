### Just Scripts

By adding a shebang line to the top of a `justfile` and making it executable, `just` can be used as an interpreter for scripts:

````sh
$ cat > script <<EOF
#!/usr/bin/env just --justfile

foo:
  echo foo
EOF
$ chmod +x script
$ ./script foo
echo foo
foo
````

When a script with a shebang is executed, the system supplies the path to the script as an argument to the command in the shebang. So, with a shebang of `#!/usr/bin/env just --justfile`, the command will be `/usr/bin/env just --justfile PATH_TO_SCRIPT`.

With the above shebang, `just` will change its working directory to the location of the script. If you’d rather leave the working directory unchanged, use `#!/usr/bin/env just --working-directory . --justfile`.

Note: Shebang line splitting is not consistent across operating systems. The previous examples have only been tested on macOS. On Linux, you may need to pass the `-S` flag to `env`:

````make
#!/usr/bin/env -S just --justfile

default:
  echo foo
````