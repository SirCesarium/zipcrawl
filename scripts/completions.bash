# Source AFTER: eval "$(COMPLETE=bash zipcrawl)"
_zipcrawl_complete() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local arch="${COMP_WORDS[1]}"
    arch="${arch/#\~/$HOME}"
    local cmd="${COMP_WORDS[2]}"
    if [[ $COMP_CWORD -ge 3 ]] && [[ "$cmd" =~ ^(cat|bat|x|exec)$ ]] && [[ -f "$arch" ]]; then
        COMPREPLY=($(compgen -W "$(zipcrawl "$arch" list 2>/dev/null | sed 's/^[^ ]* //')" -- "$cur"))
        return
    fi
    _clap_complete_zipcrawl
}
complete -F _zipcrawl_complete zipcrawl
