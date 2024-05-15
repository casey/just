justfile grammar
================

Justfiles are processed by a mildly context-sensitive tokenizer
and a recursive descent parser. The grammar is LL(k), for an
unknown but hopefully reasonable value of k.

tokens
------

```
BACKTICK            = `[^`]*`
INDENTED_BACKTICK   = ```[^(```)]*```
COMMENT             = #([^!].*)?$
DEDENT              = emitted when indentation decreases
EOF                 = emitted at the end of the file
INDENT              = emitted when indentation increases
LINE                = emitted before a recipe line
NAME                = [a-zA-Z_][a-zA-Z0-9_-]*
NEWLINE             = \n|\r\n
RAW_STRING          = '[^']*'
INDENTED_RAW_STRING = '''[^(''')]*'''
STRING              = "[^"]*" # also processes \n \r \t \" \\ escapes
INDENTED_STRING     = """[^("""]*""" # also processes \n \r \t \" \\ escapes
LINE_PREFIX         = @-|-@|@|-
TEXT                = recipe text, only matches in a recipe body
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

item          : alias
              | assignment
              | eol
              | export
              | import
              | module
              | recipe
              | setting

eol           : NEWLINE
              | COMMENT NEWLINE

alias         : 'alias' NAME ':=' NAME

assignment    : NAME ':=' expression eol

export        : 'export' assignment

setting       : 'set' 'allow-duplicate-recipes' boolean?
              | 'set' 'allow-duplicate-variables' boolean?
              | 'set' 'dotenv-filename' ':=' string
              | 'set' 'dotenv-load' boolean?
              | 'set' 'dotenv-path' ':=' string
              | 'set' 'export' boolean?
              | 'set' 'fallback' boolean?
              | 'set' 'ignore-comments' boolean?
              | 'set' 'positional-arguments' boolean?
              | 'set' 'quiet' boolean?
              | 'set' 'shell' ':=' '[' string (',' string)* ','? ']'
              | 'set' 'tempdir ':=' string
              | 'set' 'windows-powershell' boolean?
              | 'set' 'windows-shell' ':=' '[' string (',' string)* ','? ']'

import        : 'import' '?'? string?

module        : 'mod' '?'? NAME string?

boolean       : ':=' ('true' | 'false')

expression    : 'if' condition '{' expression '}' 'else' '{' expression '}'
              | value '/' expression
              | value '+' expression
              | value

condition     : expression '==' expression
              | expression '!=' expression

value         : NAME '(' sequence? ')'
              | BACKTICK
              | INDENTED_BACKTICK
              | NAME
              | string
              | '(' expression ')'

string        : STRING
              | INDENTED_STRING
              | RAW_STRING
              | INDENTED_RAW_STRING

sequence      : expression ',' sequence
              | expression ','?

recipe        : attributes* '@'? NAME parameter* variadic? ':' dependency* body?

attributes    : '[' attribute* ']' eol

attribute     : NAME ( '(' string ')' )?

parameter     : '$'? NAME
              | '$'? NAME '=' value

variadic      : '*' parameter
              | '+' parameter

dependency    : NAME
              | '(' NAME expression* ')'

body          : INDENT line+ DEDENT

line          : LINE LINE_PREFIX? (TEXT | interpolation)+ NEWLINE
              | NEWLINE

interpolation : '{{' expression '}}'
```
