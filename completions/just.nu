def "nu-complete just" [context: string] {
    let tree = (^just --dump --unstable --dump-format json | from json)
    let tokens = ($context | split words)
    let current = if ($tokens | is-empty) { "" } else { $tokens | last }
    let previous = if ($tokens | length) <= 2 { [] } else { $tokens | skip 1 | drop 1 }
    let recipes = (^just --summary | split row " ")

    mut recipe_parts = []
    for word in $previous {
        $recipe_parts = ($recipe_parts | append $word)
        let candidate = ($recipe_parts | str join "::")

        if $candidate in $recipes {
            return []
        }
    }

    mut node = $tree
    for word in $previous {
        if (($node.modules | columns) | any {|name| $name == $word }) {
            $node = ($node.modules | get $word)
        } else {
            return []
        }
    }

    let names = (($node.modules | columns) ++ ($node.recipes | columns))

    $names
    | where {|name| $name | str starts-with $current }
    | each {|value| {value: $value description: ""} }
}

# Just: A Command Runner
export extern "just" [
    ...recipe: string@"nu-complete just", # Recipe(s) to run, may be with argument(s)
]
