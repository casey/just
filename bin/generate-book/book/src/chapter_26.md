### Strings

Double-quoted strings support escape sequences:

````make
string-with-tab             := "\t"
string-with-newline         := "\n"
string-with-carriage-return := "\r"
string-with-double-quote    := "\""
string-with-slash           := "\\"
string-with-no-newline      := "\
"
````

````sh
$ just --evaluate
"tring-with-carriage-return := "
string-with-double-quote    := """
string-with-newline         := "
"
string-with-no-newline      := ""
string-with-slash           := "\"
string-with-tab             := "     "
````

Strings may contain line breaks:

````make
single := '
hello
'

double := "
goodbye
"
````

Single-quoted strings do not recognize escape sequences:

````make
escapes := '\t\n\r\"\\'
````

````sh
$ just --evaluate
escapes := "\t\n\r\"\\"
````

Indented versions of both single- and double-quoted strings, delimited by triple single- or triple double-quotes, are supported. Indented string lines are stripped of leading whitespace common to all non-blank lines:

````make
# this string will evaluate to `foo\nbar\n`
x := '''
  foo
  bar
'''

# this string will evaluate to `abc\n  wuv\nbar\n`
y := """
  abc
    wuv
  xyz
"""
````

Similar to unindented strings, indented double-quoted strings process escape sequences, and indented single-quoted strings ignore escape sequences. Escape sequence processing takes place after unindentation. The unindention algorithm does not take escape-sequence produced whitespace or newlines into account.