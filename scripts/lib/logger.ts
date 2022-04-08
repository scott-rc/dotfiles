import { log } from "./deps.ts";
import { args } from "./args.ts";

await log.setup({
  handlers: {
    console: new log.handlers.ConsoleHandler("DEBUG"),
  },
  loggers: {
    default: {
      level: args.debug ? "DEBUG" : "INFO",
      handlers: ["console"],
    },
  },
});
