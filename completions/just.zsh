#compdef just
source <(JUST_COMPLETE=zsh just)
if [ "$funcstack[1]" = "_just" ]; then
  _clap_dynamic_completer_just "$@"
fi
