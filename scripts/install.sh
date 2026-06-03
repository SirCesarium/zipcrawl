#!/usr/bin/env bash
set -euo pipefail

REPO="SirCesarium/zipcrawl"
VERSION="${ZIPCRAWL_VERSION:-latest}"
BIN_DIR="${ZIPCRAWL_BIN_DIR:-$HOME/.local/bin}"

detect() {
    local arch
    arch="$(uname -m)"
    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="arm64" ;;
        *) return 1 ;;
    esac

    local os ext abi
    os="$(uname -s)"
    case "$(echo "$os" | tr '[:upper:]' '[:lower:]')" in
        linux)   os="linux";    ext="";    abi="-musl" ;;
        darwin)  os="macos";    ext="";    abi="" ;;
        mingw*|msys*|cygwin*)   os="windows"; ext=".exe"; abi="" ;;
        *) return 1 ;;
    esac

    printf '%s\n' "$os" "$arch" "$ext" "zipcrawl-${os}-${arch}${abi}${ext}"
}

download_binary() {
    local os arch ext name
    { read -r os; read -r arch; read -r ext; read -r name; } < <(detect)

    local url
    if [ "$VERSION" = "latest" ]; then
        url="https://github.com/$REPO/releases/latest/download/$name"
    else
        url="https://github.com/$REPO/releases/download/$VERSION/$name"
    fi

    local tmp; tmp="$(mktemp -d)"
    local dest="$tmp/zipcrawl$ext"

    echo "-> Downloading zipcrawl ..." >&2
    if command -v curl &>/dev/null; then
        curl -fsSL "$url" -o "$dest"
    elif command -v wget &>/dev/null; then
        wget -q "$url" -O "$dest"
    else
        echo "curl or wget required" >&2; exit 1
    fi

    chmod +x "$dest"
    mkdir -p "$BIN_DIR"
    cp "$dest" "$BIN_DIR/zipcrawl"
    rm -rf "$tmp"
    echo "$BIN_DIR/zipcrawl"
}

install_via_cargo() {
    if ! command -v cargo &>/dev/null; then
        echo "-> Install Rust (https://rustup.rs) then: cargo install zipcrawl" >&2
        exit 1
    fi
    echo "-> Compiling via cargo install ..." >&2
    cargo install zipcrawl
    command -v zipcrawl
}

install_completions() {
    local bin="$1"

    if command -v fish &>/dev/null; then
        local fish_dir="${XDG_CONFIG_HOME:-$HOME/.config}/fish/completions"
        mkdir -p "$fish_dir"
        cat > "$fish_dir/zipcrawl.fish" << 'FISH'
complete -c zipcrawl -n "not __fish_seen_subcommand_from tree t cat bat list ls l find fd f grep g x exec diff d completions" -a "tree t cat bat list ls l find fd f grep g x exec diff d completions"
complete -c zipcrawl -n "__fish_seen_subcommand_from cat bat x exec" -f -a "(zipcrawl (string replace -r '^~' \$HOME -- (commandline -opc)[2]) list 2>/dev/null | string replace -r '^\S+\s+' '')"
FISH
    fi

    if command -v zsh &>/dev/null; then
        local zsh_dir="${ZIPCRAWL_ZSH_COMP:-${XDG_DATA_HOME:-$HOME/.local/share}/zsh/site-functions}"
        mkdir -p "$zsh_dir"
        cat > "$zsh_dir/_zipcrawl" << 'ZSH'
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
        archive) _files ;;
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
                cat|bat|x|exec) _zipcrawl_list_zip_files ;;
                tree|t)
                    _arguments '(-d --depth)'{-d+,--depth=}'[Depth]:depth' '(-s --sizes)'{-s,--sizes}'[Show sizes]'
                    ;;
                list|ls|l)
                    _arguments '(-s --sizes)'{-s,--sizes}'[Show sizes]'
                    ;;
                diff|d)
                    _arguments '(-b --base)'{-b+,--base=}'[Base archive]: :_files' \
                        '(-m --mode)'{-m+,--mode=}'[Detail level]:(default structure stats full)' \
                        '(-i --include)'{-i+,--include=}'[Include patterns]' \
                        '(-e --exclude)'{-e+,--exclude=}'[Exclude patterns]' \
                        '(-q --quiet)'{-q,--quiet}'[Quiet mode]'
                    ;;
                completions)
                    _arguments ':shell:(bash elvish fish powershell zsh)'
                    ;;
            esac
            ;;
    esac
}
_zipcrawl
ZSH

        local rc="${ZDOTDIR:-$HOME}/.zshrc"
        if [ -f "$rc" ]; then
            local marker='# zipcrawl completions'
            if ! grep -qsF "$marker" "$rc" 2>/dev/null; then
                {
                    echo ""
                    echo "$marker"
                    echo "source \"$zsh_dir/_zipcrawl\""
                    echo "compdef _zipcrawl zipcrawl"
                } >> "$rc"
            fi
        fi
    fi

    if command -v bash &>/dev/null; then
        local rc="${HOME}/.bashrc"
        if [ ! -f "$rc" ]; then rc="${HOME}/.bash_profile"; fi
        if [ -f "$rc" ]; then
            local marker='# zipcrawl completions'
            if ! grep -qsF "$marker" "$rc" 2>/dev/null; then
                cat >> "$rc" << 'BASHEOF'

# zipcrawl completions
eval "$(COMPLETE=bash zipcrawl)"
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
BASHEOF
            fi
        fi
    fi
}

ask_nerdfonts() {
    local hr
    hr="$(printf '─%.0s' {1..50})"
    echo "$hr" >&2
    echo "  󰉋      󰈔  — can you see these icons?" >&2
    echo "  (NerdFont adds file-type icons to list/tree/bat output)" >&2
    echo "  Type 'y' → cargo install zipcrawl (compiles from source)" >&2
    echo "  Type 'N' → download prebuilt binary (no icons, faster)" >&2
    echo "$hr" >&2
    local ans
    read -r -p "  Want to add Nerd Fonts Feature? (n): " ans < /dev/tty || return 1
    case "$ans" in
        [yY]|[yY][eE][sS]) return 0 ;;
        *) return 1 ;;
    esac
}

main() {
    local bin

    if ask_nerdfonts; then
        bin="$(install_via_cargo)"
    elif detect &>/dev/null; then
        bin="$(download_binary)"
        echo "-> Installed to $bin" >&2
        echo "-> Add to PATH: export PATH=\"\$PATH:$BIN_DIR\"" >&2
    else
        echo "-> No prebuilt binary for $(uname -m) / $(uname -s)" >&2
        bin="$(install_via_cargo)"
    fi

    install_completions "$bin"
    echo "-> Completions installed for fish/zsh/bash" >&2
}

main
