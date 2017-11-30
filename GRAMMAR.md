justfile grammar
================

Justfiles are processed by a mildly context-sensitive tokenizer
and a recursive descent parser. The grammar is mostly LL(1),
although an extra token of lookahead is used to distinguish between
export assignments and recipes with parameters.

tokens
------

```
BACKTICK   = `[^`\n\r]*`
COMMENT    = #([^!].*)?$
DEDENT     = emitted when indentation decreases
EOF        = emitted at the end of the file
INDENT     = emitted when indentation increases
LINE       = emitted before a recipe line
NAME       = [a-zA-Z_][a-zA-Z0-9_-]*
NEWLINE    = \n|\r\n
RAW_STRING = '[^'\r\n]*'
STRING     = "[^"]*" # also processes \n \r \t \" \\ escapes
TEXT       = recipe text, only matches in a recipe body
```

grammar syntax
--------------

```
|   alternation
()  grouping
_?  option (0 or 1 times)
_*  repetition (0 or more times)
_+  repetition (1 or more times)
```

grammar
-------

```
justfile      : item* EOF

item          : recipe
              | assignment
              | export
              | eol

eol           : NEWLINE
              | COMMENT NEWLINE

assignment    : NAME '=' expression eol

export        : 'export' assignment

expression    : value '+' expression
              | value

value         : STRING
              | RAW_STRING
              | NAME
              | BACKTICK

recipe        : '@'? NAME parameter* ('+' parameter)? ':' dependencies? body?

parameter     : NAME
              | NAME '=' STRING
              | NAME '=' RAW_STRING

dependencies  : NAME+

body          : INDENT line+ DEDENT

line          : LINE (TEXT | interpolation)+ NEWLINE
              | NEWLINE

interpolation : '{{' expression '}}'
```
