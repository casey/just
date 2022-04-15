### Ignoring Errors

Normally, if a command returns a non-zero exit status, execution will stop. To continue execution after a command, even if it fails, prefix the command with `-`:

````make
foo:
  -cat foo
  echo 'Done!'
````

````sh
$ just foo
cat foo
cat: foo: No such file or directory
echo 'Done!'
Done!
````