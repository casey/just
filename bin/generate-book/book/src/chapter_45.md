### Quiet Recipes

A recipe name may be prefixed with `@` to invert the meaning of `@` before each line:

````make
@quiet:
  echo hello
  echo goodbye
  @# all done!
````

Now only the lines starting with `@` will be echoed:

````sh
$ j quiet
hello
goodbye
# all done!
````

Shebang recipes are quiet by default:

````make
foo:
  #!/usr/bin/env bash
  echo 'Foo!'
````

````sh
$ just foo
Foo!
````

Adding `@` to a shebang recipe name makes `just` print the recipe before executing it:

````make
@bar:
  #!/usr/bin/env bash
  echo 'Bar!'
````

````sh
$ just bar
#!/usr/bin/env bash
echo 'Bar!'
Bar!
````