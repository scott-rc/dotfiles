let-env STARSHIP_SHELL = "nu"
let-env STARSHIP_SESSION_KEY = (random chars -l 16)
let-env PROMPT_MULTILINE_INDICATOR = (^/opt/homebrew/bin/starship prompt --continuation)

# Does not play well with default character module.
# TODO: Also Use starship vi mode indicators?
let-env PROMPT_INDICATOR = ""

let-env PROMPT_COMMAND = {
    # jobs are not supported
    ^/opt/homebrew/bin/starship prompt --cmd-duration $env.CMD_DURATION_MS --status $env.LAST_EXIT_CODE
}

# Not well-suited for `starship prompt --right`.
# Built-in right prompt is equivalent to $fill$right_format in the first prompt line.
# Thus does not play well with default `add_newline = True`.
let-env PROMPT_COMMAND_RIGHT = {''}
