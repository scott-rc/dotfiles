# Author Slides

Write and edit slide content using Slidev markdown syntax, layouts, and animations.

## Instructions

1. **Locate talk**: discover `talks/` directories in the slides workspace. If multiple, present as AskUserQuestion options.
2. **Gather requirements**: determine what to write or change. If vague, batch clarifying questions into a single AskUserQuestion.
3. **Determine mode**: `create` if no `slides.md` or only starter template; `edit` if existing content.
4. **Delegate to `slide-writer` agent** with: talk_path, mode, requirements.
5. **Build verification**: run `cd <talk_path> && pnpm build` and check the exit code. If non-zero, surface the build errors to the user.
6. **Report**: relay slide count, file path, and build status.

## References

Read references/slidev-syntax.md for Slidev markdown syntax reference.
