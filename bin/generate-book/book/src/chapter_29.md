### Command Evaluation Using Backticks

Backticks can be used to store the result of commands:

````make
localhost := `dumpinterfaces | cut -d: -f2 | sed 's/\/.*//' | sed 's/ //g'`

serve:
  ./serve {{localhost}} 8080
````

Indented backticks, delimited by three backticks, are de-indented in the same manner as indented strings:

````make
# This backtick evaluates the command `echo foo\necho bar\n`, which produces the value `foo\nbar\n`.
stuff := ```
    echo foo
    echo bar
  ```
````

See the [Strings](#strings) section for details on unindenting.

Backticks may not start with `#!`. This syntax is reserved for a future upgrade.