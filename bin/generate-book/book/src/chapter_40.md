### Changing the Working Directory in a Recipe

Each recipe line is executed by a new shell, so if you change the working directory on one line, it won’t have an effect on later lines:

````make
foo:
  pwd    # This `pwd` will print the same directory…
  cd bar
  pwd    # …as this `pwd`!
````

There are a couple ways around this. One is to call `cd` on the same line as the command you want to run:

````make
foo:
  cd bar && pwd
````

The other is to use a shebang recipe. Shebang recipe bodies are extracted and run as scripts, so a single shell instance will run the whole thing, and thus a `pwd` on one line will affect later lines, just like a shell script:

````make
foo:
  #!/usr/bin/env bash
  set -euxo pipefail
  cd bar
  pwd
````