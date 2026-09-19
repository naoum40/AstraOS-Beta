# ╔══════════════════════════════════════════════════════════════════════╗
# ║                    AstraOS — Bash configuration                       ║
# ║                          Default user: astra                            ║
# ╚══════════════════════════════════════════════════════════════════════╝
#
# This file is copied to /etc/skel/.bashrc by the ISO build, then to
# ~/.bashrc on first login of the default "astra" user.

# ── Shell options ────────────────────────────────────────────────────────
[ -z "$PS1" ] && return
shopt -s checkwinsize
shopt -s histappend
shopt -s cdspell
shopt -s dirspell
shopt -s globstar 2>/dev/null

# ── History ──────────────────────────────────────────────────────────────
HISTCONTROL=ignoreboth:erasedups
HISTSIZE=10000
HISTFILESIZE=20000
HISTTIMEFORMAT="%F %T  "

# ── PATH (Rust toolchain + user bins + paru) ─────────────────────────────
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$HOME/.local/share/paru/bin:$HOME/.local/share/gem/ruby/bin:$PATH"

# ── Editor & pager ───────────────────────────────────────────────────────
export EDITOR=nvim
export VISUAL=nvim
export PAGER=less
export LESS="-R -F -X -i"
export MANPAGER="sh -c 'col -bx | bat -l man -p' 2>/dev/null || less"

# ── AstraOS branding env ────────────────────────────────────────────────
export ASTRAOS_VERSION="0.1.0"
export ASTRAOS_EDITION="Core"

# ── Aliases ──────────────────────────────────────────────────────────────
# Modern replacements — fall back to classic tools if not installed.
if command -v eza >/dev/null 2>&1; then
    alias ll='eza -la --git --group-directories-first'
    alias l='eza -l --git --group-directories-first'
    alias la='eza -a --git --group-directories-first'
    alias lt='eza -T --level=2'
else
    alias ll='ls -la --color=auto'
    alias l='ls -l --color=auto'
    alias la='ls -A --color=auto'
fi

if command -v bat >/dev/null 2>&1; then
    alias cat='bat --paging=never --style=plain'
    alias batcat='bat'
else
    alias cat='cat'
fi

if command -v rg >/dev/null 2>&1; then
    alias grep='rg'
fi

if command -v fd >/dev/null 2>&1; then
    alias find='fd'
fi

alias ..='cd ..'
alias ...='cd ../..'
alias ....='cd ../../..'
alias cp='cp -iv'
alias mv='mv -iv'
alias rm='rm -Iv --preserve-root'
alias mkdir='mkdir -pv'
alias chmod='chmod -c'
alias chown='chown -c'
alias df='df -h'
alias du='du -h'
alias free='free -h'
alias ports='sudo ss -tulpn'
alias ports6='sudo ss -6tulpn'

# ── AstraOS-specific aliases ─────────────────────────────────────────────
alias astra-version='echo "AstraOS $ASTRAOS_EDITION v$ASTRAOS_VERSION"'
alias astra-update='sudo pacman -Syu && paru -Syu'
alias astra-clean='sudo pacman -Rns $(pacman -Qtdq) 2>/dev/null; paru -Sc'

# Source /etc/astra/aliases if maintained by the system image
[ -f /etc/astra/aliases ] && . /etc/astra/aliases

# ── Color prompt with AstraOS branding ───────────────────────────────────
# Magenta ✦ symbol (#FF2D55-ish) + classic 4-color prompt.
ASTRA_COLOR='\[\e[35m\]'   # magenta
PATH_COLOR='\[\e[34m\]'    # blue
GIT_COLOR='\[\e[33m\]'     # yellow
USER_COLOR='\[\e[32m\]'   # green
RESET='\[\e[0m\]'

# Lightweight git branch (no external deps)
__astra_git_branch() {
    local b
    b=$(git symbolic-ref --short HEAD 2>/dev/null) \
        || b=$(git rev-parse --short HEAD 2>/dev/null) \
        || return
    printf ' (%s)' "$b"
}

PS1="${ASTRA_COLOR}✦${RESET} ${USER_COLOR}\u${RESET}@${PATH_COLOR}\h${RESET}:${PATH_COLOR}\w${RESET}${GIT_COLOR}\$(__astra_git_branch)${RESET}\n\$ "

# ── Starship (preferred prompt when installed) ─────────────────────────
if command -v starship >/dev/null 2>&1; then
    eval "$(starship init bash)"
fi

# ── bash completion (if installed) ──────────────────────────────────────
if [ -f /usr/share/bash-completion/bash_completion ]; then
    . /usr/share/bash-completion/bash_completion
fi

# ── Welcome banner on login shell ───────────────────────────────────────
if [ -n "$PS1" ] && [ -z "$ASTRA_NO_BANNER" ]; then
    printf '\n'
    printf '\e[35m   ╦   ╦╔╗╔╔╦╗╔═╗╦╔╗╔╦\e[0m\n'
    printf '\e[35m   ║   ║║║║ ║ ║  ║║║║║\e[0m  \e[90mAstraOS\e[0m \e[1mv%s\e[0m (\e[1m%s\e[0m)\n' "$ASTRAOS_VERSION" "$ASTRAOS_EDITION"
    printf '\e[35m   ╩═╝╩╝╚╝ ╩ ╚═╝╩╝╚╝╩\e[0m  \e[90m© 2026 Astra Corporation\e[0m\n'
    printf '\n'
fi
