### Variables and Substitution

Variables, strings, concatenation, and substitution using `{{…}}` are supported:

````make
version := "0.2.7"
tardir  := "awesomesauce-" + version
tarball := tardir + ".tar.gz"

publish:
  rm -f {{tarball}}
  mkdir {{tardir}}
  cp README.md *.c {{tardir}}
  tar zcvf {{tarball}} {{tardir}}
  scp {{tarball}} me@server.com:release/
  rm -rf {{tarball}} {{tardir}}
````

#### Escaping `{{`

To write a recipe containing `{{`, use `{{{{`:

````make
braces:
  echo 'I {{{{LOVE}} curly braces!'
````

(An unmatched `}}` is ignored, so it doesn’t need to be escaped.)

Another option is to put all the text you’d like to escape inside of an interpolation:

````make
braces:
  echo '{{'I {{LOVE}} curly braces!'}}'
````

Yet another option is to use `{{ "{{" }}`:

````make
braces:
  echo 'I {{ "{{" }}LOVE}} curly braces!'
````