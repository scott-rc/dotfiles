# Basic zsh config

# History
HISTFILE=~/.zsh_history
HISTSIZE=10000
SAVEHIST=10000
setopt HIST_IGNORE_DUPS
setopt SHARE_HISTORY

# Basic options
setopt AUTO_CD
setopt INTERACTIVE_COMMENTS

# Key bindings
bindkey -e

# Completion
autoload -Uz compinit && compinit

# Prompt
PROMPT='%~ %# '
