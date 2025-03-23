#!/usr/bin/env bash

# locate cygpath if available
CYGPATH=$(command -v cygpath 2>/dev/null)
if [[ "$OSTYPE" == "cygwin" || "$OSTYPE" == "msys" ]] && [[ -z "$CYGPATH" ]]; then
  echo "cygpath.exe not found! Ensure it's installed."
  exit 1
fi

# cd upwards to the justfile
while [[ ! -e justfile ]]; do
  if [[ $PWD = / ]] || [[ $PWD = $JUSTSTOP ]] || [[ -e juststop ]]; then
    echo 'No justfile found.'
    exit 1
  fi
  cd ..
done

# prefer gmake if it exists
if command -v gmake > /dev/null; then
  MAKE=gmake
else
  MAKE=make
fi

declare -a RECIPES
for ARG in "$@"; do
  test $ARG = '--' && shift && break
  # convert paths if needed (Cygwin/MSYS2)
  if [[ "$OSTYPE" == "cygwin" || "$OSTYPE" == "msys" ]]; then
    ARG=$($CYGPATH "$ARG")
  fi
  RECIPES+=($ARG) && shift
done

# export arguments after '--' so they can be used in recipes
I=0
for ARG in "$@"; do
    export ARG$I=$ARG
    I=$((I + 1))
done

# go!
exec $MAKE MAKEFLAGS='' --always-make --no-print-directory -f justfile ${RECIPES[*]}
