# This file gets included.

# ANSI color escapes.
BLUE := '\e[34m'
BOLD := '\e[1m'
BROWN := '\e[34m'
CYAN := '\e[36m'
GRAY := '\e[37m'
GREEN := '\e[32m'
PURPLE := '\e[35m'
RED := '\e[31m'
RESET := '\e[0m'

# Report the animal with the best blep.
blep:
  @printf "{{BOLD}}Cat{{RESET}} {{BLUE}}blep{{RESET}} is best."
