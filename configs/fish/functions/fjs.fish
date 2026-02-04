function fjs --description "Sort JSON file keys alphabetically"
    argparse 'u/unrestricted' -- $argv
    or return

    set -l unrestricted_flag
    if set -q _flag_unrestricted
        set unrestricted_flag -u
    end

    # 1. Find/select JSON file with fzf
    set -l file (fd --type f --extension json $unrestricted_flag | fzf_files --query "$argv")
    or return
    test -z "$file"; and return 0

    # 2. Detect indentation (spaces vs tabs, count)
    set -l indent "2 spaces"  # default
    set -l first_indent (grep -m1 '^[[:space:]]' "$file")
    if string match -q '	*' "$first_indent"
        set indent "tabs"
    else if string match -qr '^( +)' "$first_indent"
        set -l spaces (string match -r '^( +)' "$first_indent")[2]
        set indent (string length "$spaces")" spaces"
    end

    # 3. Sort with Claude (using Sonnet for speed/cost)
    set -l tmpfile (mktemp)
    claude -p --model sonnet "Sort all object keys alphabetically in this JSON/JSONC file.

Rules:
- Sort keys at ALL nesting levels
- Preserve comments exactly where they are relative to their keys
- Preserve trailing commas if present
- Use $indent indentation
- Output ONLY the raw file contents
- Do NOT wrap in markdown code blocks
- Do NOT add any explanation" < "$file" | tee "$tmpfile"

    # 4. Validate and replace
    if test $status -eq 0 -a -s "$tmpfile"
        # Basic sanity check: should start with { or [
        if head -c1 "$tmpfile" | command grep -qE '[\[{]'
            mv "$tmpfile" "$file"
            echo "Sorted keys in $file"
        else
            rm -f "$tmpfile"
            echo "fjs: Claude output validation failed"
            return 1
        end
    else
        rm -f "$tmpfile"
        echo "fjs: Failed to sort $file"
        return 1
    end
end
