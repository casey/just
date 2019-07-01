# Quiet Recipes

A recipe name may be prefixed with '@' to invert the meaning of '@' before each line:

```make
@quiet:
  echo hello
  echo goodbye
  @# all done!
```

Now only the lines starting with '@' will be echoed:

```sh
$ j quiet
hello
goodbye
# all done!
```
