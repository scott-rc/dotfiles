if not status is-interactive
    return
end

alias n=npm
abbr --add nb npm run build
abbr --add ncl npm run clean
abbr --add nd npm run dev
abbr --add nf npm run fmt
abbr --add ni npm install
abbr --add nid npm install -D
abbr --add nig npm install -g
abbr --add nl npm run lint
abbr --add nr npm run
abbr --add nrm npm rm
abbr --add ns npm run start
abbr --add nt npm run test
abbr --add nx npx

alias p=pnpm
abbr --add pb pnpm run build
abbr --add pcl pnpm run clean
abbr --add pd pnpm run dev
abbr --add pf pnpm run fmt
abbr --add pi pnpm install
abbr --add pid pnpm install -D
abbr --add pl pnpm run lint
abbr --add pr pnpm run
abbr --add prs pnpm run start
abbr --add pt pnpm run test
abbr --add px pnpm x

alias y=yarn
abbr --add ya yarn add
abbr --add yad yarn add -D
abbr --add yb yarn build
abbr --add yc yarn clean
abbr --add yd yarn dev
abbr --add yi yarn install
abbr --add yl yarn lint
abbr --add yr yarn run
abbr --add yrm yarn remove
abbr --add ys yarn start
abbr --add yt yarn test
abbr --add yu yarn upgrade-interactive --latest
abbr --add yw yarn workspace
abbr --add yws yarn workspaces

abbr --add vt vitest
