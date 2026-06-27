# Just — Feature Inventory

A compact, source-derived map of every user-facing feature, grouped into clusters.
Derived from `src/` + `tests/` only (not the README). **(unstable)** marks features
gated behind `--unstable` / `JUST_UNSTABLE=1` / `set unstable`.

The clusters below fall into four arcs: **the language** (1–7), **the execution
environment** (8–10), **organization & reuse** (11–14), and **the CLI / tooling**
(15–20). The two big recent additions — **lists/typed values (§13)** and **cached
recipes (§14)** — are both currently unstable.

---

## 1. Invocation & justfile discovery

- **Upward search**: from the invocation dir, ascend ancestors for `justfile`/`.justfile` (case-insensitive), stopping at the closest; multiple candidates in one dir is an error.
- **`--justfile`/`-f PATH`**, **`-f -`** to read the justfile from **stdin**.
- **`--working-directory`/`-d`** (requires `--justfile`); **search-directory positional** (`just foo/`, `../`, `../recipe`) — conflicts with `-f`/`-d`.
- **`--global-justfile`/`-g`** (XDG / `~/.config/just` / `~/justfile`); **`--justfile-name NAME`** (custom names); **`--ceiling DIR`** (don't ascend past).
- **`set fallback`**: on unknown recipe/submodule, re-search the parent dir's justfile and retry (chains; invocation/search-dir modes only).
- **Markdown justfiles**: a `--justfile *.md` is **tangled** (code blocks extracted) into a temp justfile.
- **Project-root detection** via VCS markers (`.git`, `.hg`, `.svn`, `.bzr`, `_darcs`) — used by stdin/global/`--init`.

## 2. Recipes

- **Recipe** = `name params: deps` header + indented body of lines; lines may carry `{{ expression }}` interpolations.
- **Default recipe**: first recipe runs when none named; **`[default]`** attribute overrides (≤1 per module; default recipe must take no required args).
- **Private recipes**: leading `_` name or **`[private]`** (hidden from listings).
- **Doc comments**: preceding `#` comment or **`[doc("…")]`**; shown in `--list`, `--show`, `--explain`.
- **Line prefixes (sigils)**: `@` quiet/no-echo, `-` ignore-error, `?` guard (only with `set guards`; exit 1 halts silently); combinable (`@-`, `-@`).
- **Comments**: `#` body lines; `set ignore-comments` strips them instead of passing to shell.
- **Line continuation**: trailing `\` joins body lines and dependency lists.

## 3. Parameters & command-line arguments

- **Parameter kinds**: required (`x`), default-valued (`x="v"`, any expression), variadic `+x` (≥1), variadic `*x` (≥0, must be last).
- **Earlier-param references**: a default may reference earlier parameters but not later ones.
- **Exported params**: `$x` (or `set export`) injects the parameter into the recipe environment.
- **Parameter shadowing**: parameters shadow same-named global variables.
- **`[arg(...)]` — parameters as CLI options/flags** (major recent feature):
  - `long`/`long='name'`, `short`/`short='x'` (auto-derive from name/first char) expose `--opt`/`-o`.
  - `value=EXPR` (option supplies a fixed evaluated value), `flag` **(unstable)** (boolean `true`/empty), `multiple` **(unstable)** (repeatable → list / count).
  - `pattern='regex'` (anchored full-match validation, const-context), `help='…'` (per-arg help).
  - Combined shorts `-abc` (value-taking short last); `--opt=val`/`-o val`; options interleave with positionals; `--` stops option parsing.
- **CLI overrides**: positional `name=value` and repeatable `--set VAR VALUE` (positional wins over `--set`).
- **Multiple recipes per invocation** run sequentially, each with its own args; `--one` forbids more than one.

## 4. Dependencies

- **Prior deps** (`recipe: a b`) run before the body; **subsequent deps** (`recipe: && c d`) run after; mixable.
- **Dependency arguments**: `(dep arg1 arg2)`, arg count checked against the target's range.
- **Run-once**: each dep runs at most once per invocation (subsequents may re-run after running as a prior).
- **Cross-module deps**: deps can target `foo::bar`; run in the callee module's directory.
- **`[parallel]`**: run a recipe's dependencies concurrently.
- **Mapped dependencies (unstable)**: `*(dep *list)` fans out, running `dep` once per list element (needs `set lists`; exactly one starred arg).
- **`--no-deps`/`--no-dependencies`**: skip prior + subsequent deps; **recursion limits** on nested deps and function calls.

## 5. Execution models

- **Default**: each body line is a separate `sh -c` command — state does **not** persist across lines; stops on first failure.
- **Shebang recipes** (`#!…`): whole body written to a temp file and run as one script (state persists, continues past failures); cmd/Windows omit the shebang line.
- **`[script]` / `[script('interp', args…)]`** and **`set default-script`**: run the whole body via an interpreter (default `sh -eu` via `set script-interpreter`); **`[shell]`** forces line-by-line shell mode.
- **`[extension('.ext')]`**: temp-file extension for shebang/script recipes.
- **Shell configuration**: `set shell := [cmd, args…]` (default `sh -cu`), CLI `--shell` / `--shell-arg` / `--clear-shell-args`; **`set windows-shell`** / **`set windows-powershell`** (Windows only); `ShellKind` detects cmd/pwsh to pick file extension & arg form.

## 6. The expression language

- **String literals (6 forms)**: `'raw'`, `"cooked"` (escapes), triple `'''…'''` / `"""…"""` (indented, leading-newline + common-indent stripped), `` `backtick` `` (run in shell, capture stdout), triple-backtick (indented command string).
- **String prefixes**: `x'…'` shell-expansion (`$VAR`, `~`; no space before quote), `f'…'` format strings (`{{ expr }}` splicing, nestable).
- **Escapes** (cooked only): `\n \r \t \\ \" \<newline>` and `\u{…}` (1–6 hex, ≤10FFFF).
- **Operators**: `+` concat, `/` path-join (binary, unary-prefix `/a`), `++` list-concat **(unstable)**; comparison `==` `!=` `=~` `!~` (regex); logical `&&` `||` and `!` **(unstable)** — return operands, short-circuit.
- **Conditionals**: `if cond { … } else { … }` with `else if` chaining; `if` without `else` and non-comparison conditions are **(unstable)**; non-taken branch never evaluated.
- **`assert(condition[, message])`**.
- **Interpolation** `{{ … }}` in recipe lines (multi-line allowed; comments inside disallowed).
- **Lexical**: leading BOM ignored; unicode-safe; non-unicode paths warn (not error).

## 7. Variables, assignments & constants

- **`x := expr`** (lazy: resolved on use); circular/self/undefined-variable detection.
- **`export x := …`**, **`set export`** (export all vars + params), **`unexport VAR`** (unset an inherited env var); `$NAME` shorthand for exported params.
- **`set allow-duplicate-variables`**: later assignment silently overrides.
- **Lazy/eager evaluation**: **`set lazy`** evaluates only assignments reachable from the invoked recipe(s); **`eager x := …`** forces a single assignment to always evaluate.
- **Built-in constants**: `HEX`/`HEXLOWER`/`HEXUPPER`, `PATH_SEP`/`PATH_VAR_SEP` (Windows-aware), terminal styles `CLEAR NORMAL BOLD ITALIC UNDERLINE INVERT HIDE STRIKETHROUGH`, and ANSI colors `BLACK…WHITE` / `BG_BLACK…BG_WHITE`.

## 8. Environment & dotenv

- **Dotenv family** (`set` + CLI flags): `dotenv-load`, `dotenv-override` (dotenv beats process env), `dotenv-required`, `dotenv-filename` (search name, default `.env`), `dotenv-path`/`-E` (explicit file), `dotenv-command` (load a command's stdout); `--no-dotenv` disables. `dotenv-command` conflicts with the others.
- **Search**: `.env` walked up the working dir's ancestors; a found `dotenv-path` short-circuits the walk; multiple filenames/paths need `set lists`.
- **`env()` / `env_var()` / `env_var_or_default()`**: read from merged dotenv + process env; accept a key list (first present wins) under lists.
- **Submodule dotenv inheritance**: child sees root `.env`, loads + merges its own over the parent.

## 9. Working directory

- **Default**: recipes run in the justfile's (or submodule's) directory.
- **`set no-cd`** / **`[no-cd]`**: run in the invocation dir instead.
- **`set working-directory := "…"`** / **`[working-directory("…")]`** (expression; relative to module; implies cd, overrides `no-cd`). `no-cd` and `working-directory` settings conflict.
- **`invocation_directory()`** / **`invocation_directory_native()`** expose the original cwd; working dir also applies to backticks and `shell()`.

## 10. Built-in functions

~80 functions; key categories:

- **System/OS**: `arch`, `os`, `os_family`, `num_cpus`, `just_executable`, `just_pid`, `is_dependency`, `recipe_name`.
- **Paths (justfile/module/source)**: `justfile`, `justfile_directory`, `module_directory`, `module_file`, `module_path`, `source_directory`, `source_file`, `invocation_directory[_native]`.
- **Standard dirs (XDG)**: `home_directory`, `cache_directory`, `config_directory`, `config_local_directory`, `data_directory`, `data_local_directory`, `executable_directory`, `runtime_directory` — each with `_dir` / `_dir_native` aliases.
- **Path manipulation**: `absolute_path`, `canonicalize`, `clean`, `join`, `parent_directory`, `extension`, `file_name`, `file_stem`, `without_extension`.
- **Filesystem**: `path_exists`, `read`, `which`, `require`.
- **String case**: `capitalize`, `lowercase`, `uppercase`, `titlecase`, `kebabcase`, `snakecase`, `shoutykebabcase`, `shoutysnakecase`, `lowercamelcase`, `uppercamelcase`.
- **String editing**: `append`, `prepend`, `replace`, `replace_regex`, `trim[_start|_end][_match|_matches]`, `quote`, `encode_uri_component`.
- **Split/join**: `split`, `join_list`.
- **Hashing**: `blake3`, `blake3_file`, `sha256`, `sha256_file`.
- **Random/IDs/time**: `choose`, `uuid`, `datetime`, `datetime_utc`.
- **Misc**: `semver_matches`, `style`, `bool`, `show`, `error`, `env[_var…]`, `shell`.

## 11. Attributes (catalog)

Syntax: stacked `[…]` lines or comma-separated `[a, b]`; single-arg shorthand `[attr: 'v']`; only `arg`, `env`, `group`, `metadata` are repeatable.

- **Docs/listing**: `[doc]`/`[doc('…')]`, `[group('…')]`, `[private]`.
- **OS gating** (enable only on OS): `[unix]`, `[windows]`, `[macos]`, `[linux]`, `[openbsd]`, `[freebsd]`, `[netbsd]`, `[dragonfly]`, `[android]`.
- **Execution**: `[confirm]`/`[confirm('prompt')]`, `[no-exit-message]`/`[exit-message]`, `[no-quiet]`, `[positional-arguments]`, `[parallel]`, `[continue('SIGINT', …)]`.
- **Environment/cwd**: `[env('K','V')]`, `[working-directory('…')]`, `[no-cd]`.
- **Scripting**: `[script]`/`[script('interp', …)]`, `[shell]`, `[extension('.ext')]`.
- **Other**: `[default]`, `[metadata(…)]`, `[arg(…)]` (see §3), `[cache(…)]` (see §14, unstable).
- **Item-kind validity**: modules accept `doc`/`group`/`private`; aliases & assignments accept `private`; the rest are recipe-only.

## 12. Modules

- **`mod foo`** / **`mod foo "path"`** / **`mod? foo`** (optional, absent if missing).
- **Auto-resolution** (no path): `foo.just`, `foo/mod.just`, `foo/justfile`, `foo/.justfile`; tilde expansion; arbitrary nesting; circular detection.
- **Invocation**: `just foo bar` (spaced) or `just foo::bar` (colon); bare `just foo` runs the module's default recipe (or lists it under `default-list`).
- **Per-module settings & directory** (submodules don't inherit `no-cd`); module `[doc]`/`[group]`/`[private]`.
- **Optional-module disabling**: a recipe depending on an absent optional module is itself disabled & hidden (transitive); directly invoking an absent module is an error.

## 13. Imports

- **`import "path"`** / **`import? "path"`** (optional): splice another file's recipes/vars/settings; relative paths, tilde expansion, recursion, diamond imports.
- **Override precedence**: importing file's items override imported ones; imported recipes are never the default.

## 14. Lists & typed values **(unstable — `set lists`)**

A major recent addition; unifies strings/booleans/lists into one `Value` (a `Vec<String>` of elements).

- **List literals** `[a, b, c]` (trailing comma; nested lists flatten/splice).
- **Booleans as values**: truthy = non-empty, **falsy = empty list `[]`**; `true` = `["true"]`, `false` = `[]` (note: `''` is truthy).
- **Element-wise operators**: `+` and `/` broadcast scalar⊗list / zip equal-length lists; `++` concatenates lists; comparison is structural (`["a","b"] != ["a b"]`).
- **List-returning functions**: `split`, `which` (`[]` when not found), `env`, `append`/`prepend` (map per element); `join_list` collapses; `show` debug-renders.
- **Typed-boolean functions**: `bool`, `path_exists`, `is_dependency`, `semver_matches` return real booleans (compose with `&&`/`||`/`!`).
- **Reaches into**: recipe parameters (bind as lists), dependency arg arity, **mapped deps** `*(dep *xs)`, `[arg(flag)]`/`[arg(multiple)]`, multi-value dotenv, exports (empty list ⇒ env var unset).
- **Guard**: scalar contexts reject multi-element values ("list value … behavior undecided"); list-feature violations are reported eagerly at compile time.

## 15. Cached recipes **(unstable — `[cache]`)**

Per-recipe caching that skips re-running when nothing relevant changed. **Script/shebang recipes only.**

- **`[cache(inputs=…, outputs=…, extra=…)]`**: `inputs` files content-hashed into the key; `outputs` must exist for a hit (and are verified to be created); `extra` is an arbitrary expression folded into the key. All optional; evaluated with recipe args in scope.
- **Cache key** = blake3 of JSON {body, exported env, interpreter, extra, input content hashes, positional args (if used), recipe path, working dir}. Invalidates on any of these; **not** on input mtime or unexported vars.
- **Storage**: `.justcache/<64-hex>.json` next to the justfile; per-entry advisory file lock (parallel-safe).
- **`--clean [PATH]`**: delete cache entries, optionally only those whose recipe path starts with `PATH`.
- **`--no-cache` / `JUST_NO_CACHE`**: bypass entirely; `--dry-run` skips all cache logic; `-v`/`-vv` print cache hits / full key.

## 16. Listing, introspection & docs subcommands

- **`--list`/`-l [MODULE]`** + `--list-heading`, `--list-prefix`, `--list-submodules`, `--group`, `--unsorted`/`-u`, `--no-aliases`, `--alias-style left|right|separate`.
- **`--summary`** (names, recurses, implies `--unstable`), **`--show`/`-s PATH`** (source), **`--usage PATH`** (CLI usage w/ options), **`--groups`**, **`--variables`**.
- **`--dump`** (`--dump-format just|json`), **`--json`** (full AST), **`--evaluate`/`--eval [VAR]`** (`--evaluate-format just|shell`).
- **`--fmt`/`--format`** (rewrites in place; `--check` prints diff & exits 1; `--indentation`), **`--init`**, **`--edit`/`-e`** (`$VISUAL`→`$EDITOR`→vim).
- **`--man`**, **`--changelog`**.

## 17. Tooling & external integration

- **`--choose`** (+ `--chooser` / `$JUST_CHOOSER`, default `fzf`): pick zero-arg recipes interactively, run selections.
- **`--command`/`-c BINARY …`**: run an arbitrary command with the justfile's working dir, `.env`, overrides & exports applied; `--shell-command` runs it through the recipe shell.
- **`--completions SHELL`** for bash/elvish/fish/nushell/powershell/zsh, plus **dynamic completion** of recipes/overrides/variables/groups/modules; **`--complete-aliases`**.

## 18. Output, logging & interaction control

- **`--dry-run`/`-n`** (conflicts `--quiet`), **`--quiet`/`-q`**, **`--verbose`/`-v`** (Taciturn→Loquacious→Grandiloquent).
- **Echo styling**: `--highlight`/`--no-highlight`, `--command-color`, `--color auto|always|never`.
- **`--timestamp`** (+ `--timestamp-format`), **`--time`** (per-recipe duration), **`--explain`** (print doc before running).
- **Confirmation**: `[confirm]` recipes prompt; `--yes` / `JUST_YES` auto-confirm.
- **Quiet layering**: `set quiet` + `@`/`[no-quiet]` + `--quiet` stack.

## 19. Errors, exit codes & signals

- **Exit codes** propagate from recipes (`exit N` → status N); rich error messages with "did you mean" suggestions.
- **Signals**: uncaught SIGINT → 130; `just` forwards signals to children; `[continue('SIG', …)]` survives listed signals; `-` (infallible) clears a caught signal; SIGINFO (BSD/macOS) reports running children.
- **`--allow-missing`**: ignore unknown-recipe/submodule errors at run time only.
- **`set no-exit-message`** / `[no-exit-message]` / `[exit-message]` control the failure message.

## 20. Settings & configuration reference

**Settings** (`set NAME := …`): `allow-duplicate-recipes`, `allow-duplicate-variables`, `default-list`, `default-script`, `dotenv-command`, `dotenv-filename`, `dotenv-load`, `dotenv-override`, `dotenv-path`, `dotenv-required`, `export`, `fallback`, `guards`, `ignore-comments`, `lazy`, `lists` *(unstable)*, `minimum-version`, `no-cd`, `no-exit-message`, `positional-arguments`, `quiet`, `script-interpreter`, `shell`, `tempdir`, `unstable`, `windows-powershell`, `windows-shell`, `working-directory`.

- **`minimum-version`**: require a minimum just version (literal `X.Y.Z` only).
- **`positional-arguments`**: pass recipe args as `$0`/`$1`/`$@` to the shell.
- **Env-var config**: nearly every CLI flag has a `JUST_*` env-var backing (e.g. `JUST_CHOOSER`, `JUST_COLOR`, `JUST_UNSTABLE`); also `--cygpath`, `--tempdir`, `--indentation`.

---

## Notes for the README rewrite

Observations from building this inventory, oriented toward restructuring the README:

- **Two big features are undocumented-by-assumption here**: lists/typed values (§13) and cached recipes (§14). Both are unstable but large; each deserves its own top-level section, clearly flagged unstable, with the *mental model* up front (for lists: "everything is a list; empty list is the only false value"; for cache: "skips re-running a script recipe unless its key inputs change").
- **The opening should state the model in one breath**: a justfile is a list of recipes; recipes are run with `just NAME args`; lines run in `sh` by default; there's an expression language for variables/args. The current feature surface is enormous, so the intro must anchor newcomers before the firehose.
- **Natural top-level grouping** that the code supports: *Language* (recipes/params/deps/expressions/variables) → *Execution* (shells, env/dotenv, working dir, signals) → *Organization* (modules, imports, aliases, groups) → *CLI & tooling* (subcommands, completions, output flags) → *Reference tables* (functions, constants, attributes, settings).
- **Three large reference tables carry most of the surface area** and should be tables, not prose: functions (§10, ~80), attributes (§11), settings (§20). The codebase already treats these as registries.
- **Cross-cutting "the same thing three ways"** is a recurring pattern worth a callout box each: quiet (`set quiet` / `@` / `--quiet` / `[no-quiet]`), working dir (setting / attribute / `no-cd`), positional-args (setting / attribute), script mode (shebang / `[script]` / `set default-script` / `[shell]`). Readers get tripped by the precedence rules.
- **Likely missing / thin in a typical README**: the `[arg]` options/flags feature (it turns recipes into real CLI programs — a headline capability), guards (`?` + `set guards`), `eager`/`lazy`, `--evaluate-format shell`, dynamic shell completion, mapped dependencies, dotenv inheritance into submodules, and the full constants list.
- **Stability legend**: a single consistent badge for unstable features (lists, cache, `++`, `flag`/`multiple`, `if`-without-`else`, non-comparison conditions) would prevent newcomers from trying things that error without `--unstable`.
