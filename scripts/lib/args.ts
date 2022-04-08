import { flags } from "./deps.ts";

const parsed = flags.parse(Deno.args);

export const args = {
  debug: !!parsed.debug,
};
