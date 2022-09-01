# This file gets included.

# ANSI color escapes.
BOLD := '\e[1m'
RESET := '\e[0m'
RED := '\e[31m'
GREEN := '\e[32m'
BROWN := '\e[34m'
BLUE := '\e[34m'
PURPLE := '\e[35m'
CYAN := '\e[36m'
GRAY := '\e[37m'

# Report the animal with the best blep.
blep:
  @printf "{{BOLD}}Cat{{RESET}} {{BLUE}}blep{{RESET}} is best."
