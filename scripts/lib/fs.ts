import { log, fs } from "./deps.ts";

export async function ensureSymlink(from: string, to: string): Promise<void> {
  const toFileInfo = await lstat(to);

  try {
    if (!toFileInfo || (await Deno.realPath(to)) !== from) {
      if (toFileInfo) {
        log.warning(`${to} already exists, moving it to ${to}.bak`);
        await fs.move(to, `${to}.bak`);
      }

      log.info(`creating symlink from ${from} -> ${to}`);
      await Deno.symlink(from, to);
    } else {
      log.debug(`symlink already exists ${from} -> ${to}`);
    }
  } catch (err) {
    if (toFileInfo?.isSymlink && err instanceof Deno.errors.NotFound) {
      // we errored while trying to `Deno.realPath` a broken symlink
      log.warning(`${to} is a broken symlink... removing it`);
      await Deno.remove(to);
      return ensureSymlink(from, to);
    }
    throw err;
  }
}

export async function lstat(filePath: string): Promise<Deno.FileInfo | null> {
  try {
    return await Deno.lstat(filePath);
  } catch (err) {
    if (err instanceof Deno.errors.NotFound) {
      return null;
    }

    throw err;
  }
}

export async function exists(filePath: string): Promise<boolean> {
  return Boolean(await lstat(filePath));
}
