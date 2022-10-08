set shell := ["sh", "-c"]
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
set allow-duplicate-recipes
set positional-arguments
set dotenv-load
set export

alias s := serve

bt := '0'

export RUST_BACKTRACE_1 := bt

log := "warn"

export JUST_LOG := log

tmpdir  := `mktemp`
version := "0.2.7"
tardir  := tmpdir / "awesomesauce-" + version
foo1    := / "tmp"
foo2_3  := "a/"
tarball := tardir + ".tar.gz"

export RUST_BACKTRACE_2 := "1"
string-with-tab             := "\t"
string-with-newline         := "\n"
string-with-carriage-return := "\r"
string-with-double-quote    := "\""
string-with-slash           := "\\"
string-with-no-newline      := "\
"

# Newlines in variables
single := '
hello
'

double := "
goodbye
"
escapes := '\t\n\r\"\\'

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

for:
  for file in `ls .`; do \
    echo $file; \
  done

serve:
  touch {{tmpdir}}/file

# This backtick evaluates the command `echo foo\necho bar\n`, which produces the value `foo\nbar\n`.
stuff := ```
    echo foo
    echo bar
  ```


an_arch := trim(lowercase(justfile())) + arch()
trim_end := trim_end("99.99954%   ")
home_dir := replace(env_var('HOME') / "yep", 'yep', '')
quoted := quote("some things beyond\"$()^%#@!|-+=_*&'`")
smartphone := trim_end_match('blah.txt', 'txt')
museum := trim_start_match(trim_start(trim_end_matches('   yep_blah.txt.txt', '.txt')), 'yep_')
water := trim_start_matches('ssssssoup.txt', 's')
congress := uppercase(os())
fam := os_family()
path_1 := absolute_path('test')
path_2 := '/tmp/subcommittee.txt'
ext_z := extension(path_2)
exe_name := file_name(just_executable())
a_stem := file_stem(path_2)
a_parent := parent_directory(path_2)
sans_ext := without_extension(path_2)
camera := join('tmp', 'dir1', 'dir2', path_2)
cleaned := clean('/tmp/blah/..///thing.txt')
id__path := '/tmp' / sha256('blah') / sha256_file(justfile())
_another_var := env_var_or_default("HOME", justfile_directory())
python := `which python`

exists := if path_exists(just_executable()) =~ '^/User' { uuid() } else { 'yeah' }

foo   := if env_var("_") == "/usr/bin/env" { `touch /tmp/a_file` } else { "dummy-value" }
foo_b := if "hello" == "goodbye" { "xyz" } else { if "no" == "no" { "yep"} else { error("123") } }
foo_c := if "hello" == "goodbye" {
  "xyz"
} else if "a" == "a" {
  "abc"
} else {
  "123"
}

bar:
  @echo {{foo}}


bar2 foo_stuff:
  echo {{ if foo_stuff == "bar" { "hello" } else { "goodbye" } }}

executable:
  @echo The executable is at: {{just_executable()}}


rustfmt:
  find {{invocation_directory()}} -name \*.rs -exec rustfmt {} \;

test:
  echo "{{home_dir}}"


linewise:
  Write-Host "Hello, world!"

serve2:
  @echo "Starting server with database $DATABASE_ADDRESS on port $SERVER_PORT…"


shebang := if os() == 'windows' {
  'powershell.exe'
} else {
  '/usr/bin/env pwsh'
}

shebang:
	#!{{shebang}}
	$PSV = $PSVersionTable.PSVersion | % {"$_" -split "\." }
	$psver = $PSV[0] + "." + $PSV[1]
	if ($PSV[2].Length -lt 4) {
		$psver += "." + $PSV[2] + " Core"
	} else {
		$psver += " Desktop"
	}
	echo "PowerShell $psver"

@foo:
  echo bar

@test5 *args='':
  bash -c 'while (( "$#" )); do echo - $1; shift; done' -- "$@"

test2 $RUST_BACKTRACE="1":
  # will print a stack trace if it crashes
  cargo test


notify m="":
	keybase chat send --topic-type "chat" --channel <channel> <team> "upd(<repo>): {{m}}"

# Sample project script 2
script2 *ARGS:
    {{ python }} script2.py {{ ARGS }}

braces:
  echo 'I {{{{LOVE}} curly braces!'

_braces2:
  echo '{{'I {{LOVE}} curly braces!'}}'

_braces3:
  echo 'I {{ "{{" }}LOVE}} curly braces!'

foo2:
  -@cat foo
  echo 'Done!'

test3 target tests=path_1:
  @echo 'Testing {{target}}:{{tests}}…'
  ./test --tests {{tests}} {{target}}

test4 triple=(an_arch + "-unknown-unknown") input=(an_arch / "input.dat"):
  ./test {{triple}}

variadic $VAR1_1 VAR2 VAR3 VAR4=("a") +$FLAGS='-q': foo2 braces
  cargo test {{FLAGS}}

time:
  @-date +"%H:%S"
  -cat /tmp/nonexistant_file.txt
  @echo "finished"

# Subsequent dependencies
# https://just.systems/man/en/chapter_37.html
# To test, run `$ just -f test-suite.just b`
a:
  echo 'A!'

b: a && d
  echo 'B start!'
  just -f {{justfile()}} c
  echo 'B end!'

c:
  echo 'C!'

d:
  echo 'D!'
