function __fish_just_complete_recipes
        just --list 2> /dev/null | tail -n +2 | awk '{
        command = $1;
        args = $0;
        desc = "";
        delim = "";
        sub(/^[[:space:]]*[^[:space:]]*/, "", args);
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", args);

        if (match(args, /#.*/)) {
          desc = substr(args, RSTART+2, RLENGTH);
          args = substr(args, 0, RSTART-1);
          gsub(/^[[:space:]]+|[[:space:]]+$/, "", args);
        }

        gsub(/\+|=[`\'"][^`\'"]*[`\'"]/, "", args);
        gsub(/ /, ",", args);

        if (args != ""){
          args = "Args: " args;
        }

        if (args != "" && desc != "") {
          delim = "; ";
        }

        print command "\t" args delim desc
  }'
end

# don't suggest files right off
complete -c just -n "__fish_is_first_arg" --no-files

# complete recipes
complete -c just -a '(__fish_just_complete_recipes)'

# autogenerated completions
complete -c just -l chooser -d 'Override binary invoked by `--choose`' -r
complete -c just -l color -d 'Print colorful output' -r -f -a "{auto	'',always	'',never	''}"
complete -c just -l command-color -d 'Echo recipe lines in <COMMAND-COLOR>' -r -f -a "{black	'',blue	'',cyan	'',green	'',purple	'',red	'',yellow	''}"
complete -c just -l dump-format -d 'Dump justfile as <FORMAT>' -r -f -a "{just	'',json	''}"
complete -c just -l list-heading -d 'Print <TEXT> before list' -r
complete -c just -l list-prefix -d 'Print <TEXT> before each list item' -r
complete -c just -s f -l justfile -d 'Use <JUSTFILE> as justfile' -r -F
complete -c just -l set -d 'Override <VARIABLE> with <VALUE>' -r
complete -c just -l shell -d 'Invoke <SHELL> to run recipes' -r
complete -c just -l shell-arg -d 'Invoke shell with <SHELL-ARG> as an argument' -r
complete -c just -s d -l working-directory -d 'Use <WORKING-DIRECTORY> as working directory. --justfile must also be set' -r -F
complete -c just -s c -l command -d 'Run an arbitrary command with the working directory, `.env`, overrides, and exports set' -r
complete -c just -l completions -d 'Print shell completion script for <SHELL>' -r -f -a "{bash	'',elvish	'',fish	'',powershell	'',zsh	''}"
complete -c just -s s -l show -d 'Show information about <RECIPE>' -r
complete -c just -l dotenv-filename -d 'Search for environment file named <DOTENV-FILENAME> instead of `.env`' -r
complete -c just -s E -l dotenv-path -d 'Load <DOTENV-PATH> as environment file instead of searching for one' -r -F
complete -c just -l check -d 'Run `--fmt` in \'check\' mode. Exits with 0 if justfile is formatted correctly. Exits with 1 and prints a diff if formatting is required.'
complete -c just -l yes -d 'Automatically confirm all recipes.'
complete -c just -s n -l dry-run -d 'Print what just would do without doing it'
complete -c just -l highlight -d 'Highlight echoed recipe lines in bold'
complete -c just -l no-aliases -d 'Don\'t show aliases in list'
complete -c just -l no-deps -d 'Don\'t run recipe dependencies'
complete -c just -l no-dotenv -d 'Don\'t load `.env` file'
complete -c just -l no-highlight -d 'Don\'t highlight echoed recipe lines in bold'
complete -c just -s q -l quiet -d 'Suppress all output'
complete -c just -l shell-command -d 'Invoke <COMMAND> with the shell used to run recipe lines and backticks'
complete -c just -l clear-shell-args -d 'Clear shell arguments'
complete -c just -s u -l unsorted -d 'Return list and summary entries in source order'
complete -c just -l unstable -d 'Enable unstable features'
complete -c just -s v -l verbose -d 'Use verbose output'
complete -c just -l changelog -d 'Print changelog'
complete -c just -l choose -d 'Select one or more recipes to run using a binary chooser. If `--chooser` is not passed the chooser defaults to the value of $JUST_CHOOSER, falling back to `fzf`'
complete -c just -l dump -d 'Print justfile'
complete -c just -s e -l edit -d 'Edit justfile with editor given by $VISUAL or $EDITOR, falling back to `vim`'
complete -c just -l evaluate -d 'Evaluate and print all variables. If a variable name is given as an argument, only print that variable\'s value.'
complete -c just -l fmt -d 'Format and overwrite justfile'
complete -c just -l init -d 'Initialize new justfile in project root'
complete -c just -s l -l list -d 'List available recipes and their arguments'
complete -c just -l man -d 'Print man page'
complete -c just -l summary -d 'List names of available recipes'
complete -c just -l variables -d 'List names of variables'
complete -c just -s g -l global -d 'Use global justfile'
complete -c just -s h -l help -d 'Print help'
complete -c just -s V -l version -d 'Print version'
