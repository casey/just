def "nu-complete just" [context: string] {
    # Check if --global-justfile or -g flag is present
    let use_global = if ($context | str contains " -g ") or ($context | str contains " --global-justfile ") or ($context | str ends-with " -g") or ($context | str ends-with " --global-justfile") {
        ["--global-justfile"]
    } else {
        []
    }
    (^just ...$use_global --dump --unstable --dump-format json | from json).recipes | transpose recipe data | flatten | where {|row| $row.private == false } | select recipe doc parameters | rename value description
}

# Just: A Command Runner
export extern "just" [
    ...recipe: string@"nu-complete just", # Recipe(s) to run, may be with argument(s)
]
