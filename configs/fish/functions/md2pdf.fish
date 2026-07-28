function md2pdf --description "Convert markdown to a styled A4 PDF"
    set -l tool ~/Code/personal/dotfiles/tools/md2pdf
    if not test -x $tool/node_modules/.bin/tsx
        echo "md2pdf: dependencies not installed -- run: pnpm install -C $tool" >&2
        return 1
    end
    # --tsconfig is required away from the tool's directory: tsx resolves
    # tsconfig.json from cwd, and without it the react-jsx transform is lost.
    $tool/node_modules/.bin/tsx --tsconfig $tool/tsconfig.json $tool/src/cli.ts $argv
end
