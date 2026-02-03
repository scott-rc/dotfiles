function fzf_prompt --description "Run fzf with custom prompt and query" --argument-names PROMPT QUERY
    if test -z "$PROMPT"
        set PROMPT ""
    end

    if test -z "$QUERY"
        set QUERY ""
    end

    fzf --prompt "$PROMPT: " --query "$QUERY" --select-1 --height ~40% --reverse
end
