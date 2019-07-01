# Strings

Double-quoted strings support escape sequences:

```make
string-with-tab             := "\t"
string-with-newline         := "\n"
string-with-carriage-return := "\r"
string-with-double-quote    := "\""
string-with-slash           := "\\"
```

```sh
$ just --evaluate
"tring-with-carriage-return := "
string-with-double-quote    := """
string-with-newline         := "
"
string-with-slash           := "\"
string-with-tab             := "     "
```

Single-quoted strings do not recognize escape sequences and may contain line breaks:

```make
escapes := '\t\n\r\"\\'

line-breaks := 'hello
this
is
  a
     raw
string!
'
```

```sh
$ just --evaluate
escapes := "\t\n\r\"\\"

line-breaks := "hello
this
is
  a
     raw
string!
"
```
