#compdef zipcrawl

_zipcrawl_list_zip_files() {
    local archive="$words[2]"
    archive="${archive/#\~/$HOME}"
    if [[ -f "$archive" ]]; then
        local -a files
        files=(${(f)"$(zipcrawl "$archive" list 2>/dev/null | sed 's/^[^ ]* //')"})
        _describe 'file in archive' files
    fi
}

_zipcrawl() {
    local context state state_descr line
    typeset -A opt_args

    _arguments -C \
        '1: :->archive' \
        '2: :->cmd' \
        '*:: :->args'

    case "$state" in
        archive)
            _files
            ;;
        cmd)
            local subcmds=(
                'tree:Display directory structure'
                't:alias for tree'
                'cat:Display file contents (raw)'
                'bat:Display file contents (highlighted)'
                'list:List files and directories'
                'ls:alias for list'
                'l:alias for list'
                'find:Find files matching a pattern'
                'fd:alias for find'
                'grep:Search pattern in files'
                'g:alias for grep'
                'x:Execute command on a file'
                'exec:alias for x'
                'diff:Compare archives'
                'd:alias for diff'
                'completions:Generate completion scripts'
            )
            _describe -t commands 'subcommand' subcmds
            ;;
        args)
            case "$line[1]" in
                cat|bat|x|exec)
                    _zipcrawl_list_zip_files
                    ;;
                find|fd|f)
                    _arguments \
                        '(-p --path)'{-p+,--path=}'[Search in subdirectory]: :_zipcrawl_list_zip_files' \
                        '(-g --glob)'{-g,--glob}'[Use glob]' \
                        '(-t --entry-type)'{-t+,--entry-type=}'[Filter by type]:(f d)' \
                        ':pattern'
                    ;;
                grep|g)
                    _arguments \
                        '(-g --glob)'{-g+,--glob=}'[Glob filter]: :_zipcrawl_list_zip_files' \
                        '(-p --path)'{-p+,--path=}'[Search in subdirectory]: :_zipcrawl_list_zip_files' \
                        ':pattern'
                    ;;
                diff|d)
                    _arguments \
                        '(-b --base)'{-b+,--base=}'[Base archive]: :_files' \
                        '(-m --mode)'{-m+,--mode=}'[Detail level]:(default structure stats full)' \
                        '(-i --include)'{-i+,--include=}'[Include patterns]' \
                        '(-e --exclude)'{-e+,--exclude=}'[Exclude patterns]' \
                        '(-q --quiet)'{-q,--quiet}'[Quiet mode]'
                    ;;
                tree|t)
                    _arguments \
                        '(-d --depth)'{-d+,--depth=}'[Depth]:depth' \
                        '(-s --sizes)'{-s,--sizes}'[Show sizes]'
                    ;;
                list|ls|l)
                    _arguments '(-s --sizes)'{-s,--sizes}'[Show sizes]'
                    ;;
                completions)
                    _arguments ':shell:(bash elvish fish powershell zsh)'
                    ;;
            esac
            ;;
    esac
}

_zipcrawl
