# Command Line Options

`just` supports a number of useful command line options for listing, dumping, and debugging recipes and variable:

```sh
$ just --list
Available recipes:
  js
  perl
  polyglot
  python
  ruby
$ just --show perl
perl:
    #!/usr/bin/env perl
    print "Larry Wall says Hi!\n";
$ just --show polyglot
polyglot: python js perl sh ruby
```

Run `just --help` to see all the options.