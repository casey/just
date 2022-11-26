@include "included.just"

# The recipes in this file rely on variables defined in the
# included file!

# Report the animal with the best sploot.
sploot:
  @printf "{{BOLD}}Corgi{{RESET}} {{BLUE}}sploot{{RESET}} is best."

# Report the animal with the highest cuteness.
cute:
  @printf "{{BOLD}}The baby sea otter{{RESET}} is, at 1.0 otters, the animal rated as most cute."

help:
  @just -l

