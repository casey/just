justfile grammar
================

Justfiles are processed by a mildly context-sensitive tokenizer
and a recursive descent parser. The grammar is mostly LL(1),
although an extra token of lookahead is used to distinguish between
export assignments and recipes with arguments.

tokens
------

```
BACKTICK            = `[^`\n\r]*`
COLON               = :
COMMENT             = #([^!].*)?$
NEWLINE             = \n|\r\n
EQUALS              = =
INTERPOLATION_START = {{
INTERPOLATION_END   = }}
NAME                = [a-zA-Z_-][a-zA-Z0-9_-]*
PLUS                = +
RAW_STRING          = '[^'\r\n]*'
STRING              = "[^"]*" # also processes \n \r \t \" \\ escapes
INDENT              = emitted when indentation increases
DEDENT              = emitted when indentation decreases
LINE                = emitted before a recipe line
TEXT                = recipe text, only matches in a recipe body
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

expression    : STRING
              | RAW_STRING
              | NAME
              | BACKTICK
              | expression '+' expression

recipe        : '@'? NAME argument* ':' dependencies? body?

argument      : NAME
              | NAME '=' STRING
              | NAME '=' RAW_STRING

dependencies  : NAME+

body          : INDENT line+ DEDENT

line          : LINE (TEXT | interpolation)+ NEWLINE
              | NEWLINE

interpolation : '{{' expression '}}'
```
