# Multi-line Constructs

Recipes without an initial shebang are evaluated and run line-by-line, which means that multi-line constructs probably won't do what you want.

For example, with the following justfile:

```
conditional:
    if true; then
        echo 'True!'
    fi
```

The extra leading whitespace before the second line of the `conditional` recipe will produce a parse error:

```
$ just conditional
error: Recipe line has extra leading whitespace
  |
3 |         echo 'True!'
  |     ^^^^^^^^^^^^^^^^
```

To work around this, you can write conditionals on one line, escape newlines with slashes, or add a shebang to your recipe. Some examples of multi-line constructs are provided for reference.

### `if` statements

```make
conditional:
    if true; then echo 'True!'; fi
```

```make
conditional:
    if true; then \
        echo 'True!'; \
    fi
```

```make
conditional:
    #!/usr/bin/env sh
    if true; then
        echo 'True!'
    fi
```

### `for` loops

```make
for:
    for file in `ls .`; do echo $file; done
```

```make
for:
    for file in `ls .`; do \
        echo $file; \
    done
```

```make
for:
    #!/usr/bin/env sh
    for file in `ls .`; do
        echo $file
    done
```

### `while` loops

```make
while:
    while `server-is-dead`; do ping -c 1 server; done
```

```make
while:
    while `server-is-dead`; do \
        ping -c 1 server; \
    done
```

```make
while:
    #!/usr/bin/env sh
    while `server-is-dead`; do
        do ping -c 1 server
    done
```
