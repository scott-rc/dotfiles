import { fs, logger, Path } from "./deps.ts";

export async function ensureSymlink(from: Path, to: Path): Promise<void> {
  const toStat = await to.lstat();

  try {
    if (!toStat) {
      logger.info(`creating symlink from ${from} -> ${to}`);
      await to.parent().ensureDir();
      await from.createSymlinkTo(to);
      return;
    }

    const toRealPath = await to.realPath();
    const fromRealPath = await from.realPath();
    if (String(toRealPath) === String(fromRealPath)) {
      logger.debug(`symlink already exists ${from} -> ${to}`);
      return;
    }

    logger.warn(`${to} already exists, moving it to ${to}.bak`, {
      from: from.toString(),
      to: to.toString(),
      toRealPath: toRealPath.toString(),
      fromRealPath: fromRealPath.toString(),
    });

    await fs.move(String(to), `${to}.bak`);
    logger.info(`creating symlink from ${from} -> ${to}`);
    await to.parent().ensureDir();
    await from.createSymlinkTo(to);
  } catch (err) {
    if (toStat?.isSymlink && err instanceof Deno.errors.NotFound) {
      // `to.realPath` throws NotFound if the symlink is broken
      logger.warn(`${to} is a broken symlink... removing it`);
      await to.remove();
      return ensureSymlink(from, to);
    }

    throw err;
  }
}

export async function ensureDir(path: Path): Promise<void> {
  const stat = await path.lstat();
  if (stat?.isDirectory) return;
  await path.ensureDir();
}
