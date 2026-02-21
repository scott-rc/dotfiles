function gwt --description "Create a git worktree for a task"
    argparse 'b/branch=' 'f/from=' 'force' 'C/repo=' -- $argv
    or return 1

    # Resolve repo root
    set -l repo
    if set -q _flag_repo
        set repo $_flag_repo
    else
        set repo (command git rev-parse --show-toplevel 2>/dev/null)
        or begin
            echo "gwt: not in a git repo (use -C <path>)"
            return 1
        end
    end
    set repo (realpath $repo 2>/dev/null; or echo $repo)

    if not test -d $repo/.git
        echo "gwt: $repo is not a git repo"
        return 1
    end

    set -l wt_root $repo/.worktrees

    # Determine mode: existing branch or new branch
    set -l branch
    set -l dir_name
    set -l base

    if set -q _flag_branch
        # Existing branch mode
        set branch $_flag_branch
        set dir_name (string split -r -m1 '/' $branch)[-1]
        command git -C $repo show-ref --verify --quiet refs/heads/$branch
        or begin
            echo "gwt: branch '$branch' does not exist"
            return 1
        end
    else
        # New branch mode — build slug from argv
        if test (count $argv) -eq 0
            echo "gwt: provide task words or --branch <name>"
            return 1
        end

        set -l task_desc (string join ' ' $argv)
        set -l slug

        if status is-interactive; and command -q claude
            set slug (claude --print --model haiku \
                "Generate a short 2-4 word kebab-case git branch slug for this task. Output ONLY the slug. Task: $task_desc" \
                2>/dev/null | string trim | string lower | string replace -ra '[^a-z0-9-]+' '' | string trim --chars='-')
        end

        # Fallback: simple kebab-case
        if test -z "$slug"
            set slug (echo $task_desc | string lower | string replace -ra '[^a-z0-9]+' '-' | string trim --chars='-')
        end

        if test -z "$slug"
            echo "gwt: could not derive branch name from arguments"
            return 1
        end

        set branch sc/$slug
        set dir_name $slug

        # Determine base branch
        if set -q _flag_from
            set base $_flag_from
        else
            set base (command git -C $repo branch --show-current 2>/dev/null)
            if test -z "$base"
                echo "gwt: detached HEAD and no --from specified"
                return 1
            end
        end

        # Remove stale worktree directory if it exists
        set -l old_wt $wt_root/$dir_name
        if test -d $old_wt
            command git -C $repo worktree remove --force $old_wt 2>/dev/null
            or rm -rf $old_wt
        end

        # Handle existing branch with same name
        if command git -C $repo show-ref --verify --quiet refs/heads/$branch
            if command git -C $repo merge-base --is-ancestor $branch $base 2>/dev/null
                command git -C $repo branch -d $branch >/dev/null 2>&1
            else if set -q _flag_force
                command git -C $repo branch -D $branch >/dev/null 2>&1
            else
                echo "gwt: branch '$branch' already exists and is not merged into $base"
                return 2
            end
        end
    end

    # Create worktree
    mkdir -p $wt_root
    set -l wt_path $wt_root/$dir_name

    # Remove leftover directory (e.g., orphaned worktree)
    if test -d $wt_path; and set -q _flag_branch
        command git -C $repo worktree remove --force $wt_path 2>/dev/null
        or rm -rf $wt_path
    end

    if set -q _flag_branch
        command git -C $repo worktree add $wt_path $branch 2>&1
    else
        command git -C $repo worktree add -b $branch $wt_path $base 2>&1
    end
    or begin
        echo "gwt: failed to create worktree"
        return 1
    end

    # Copy local config files
    for f in .envrc.local .env.local CLAUDE.local.md
        if test -f $repo/$f
            cp $repo/$f $wt_path/$f
        end
    end

    # Copy .claude/**/*.local.* files preserving directory structure
    for f in (find $repo/.claude -name '*.local.*' 2>/dev/null)
        set -l rel (string replace "$repo/" '' $f)
        set -l dest_dir $wt_path/(dirname $rel)
        mkdir -p $dest_dir
        cp $f $dest_dir/
    end

    # Allow direnv if .envrc exists
    if test -f $wt_path/.envrc
        direnv allow $wt_path 2>/dev/null
    end

    # Install pnpm dependencies if applicable
    if test -f $wt_path/pnpm-lock.yaml
        echo "Running pnpm install..."
        pnpm -C $wt_path install

        # Sync lnai config if the project uses it
        if test -f $wt_path/package.json; and string match -q '*"lnai"*' <$wt_path/package.json
            echo "Running lnai sync..."
            pnpm -C $wt_path exec lnai sync
        end
    end

    echo "Worktree created: $wt_path"
    echo "Branch: $branch"

    if status is-interactive
        cd $wt_path
    else
        echo ""
        echo "  cd $wt_path"
        echo ""
        echo "cd $wt_path" | pbcopy
        echo "Copied to clipboard."
    end
end
