import { log } from "./deps.ts";
import { args } from "./args.ts";
import { bold, gray, red, yellow } from "https://deno.land/std@0.130.0/fmt/colors.ts";

class Handler extends log.handlers.ConsoleHandler {
  override format(record: log.LogRecord) {
    const msg = `${record.msg}`;

    switch (record.level) {
      case log.LogLevels.DEBUG:
        return gray(msg);
      case log.LogLevels.WARNING:
        return yellow(msg);
      case log.LogLevels.ERROR:
        return red(msg);
      case log.LogLevels.CRITICAL:
        return bold(red(msg));
      default:
        return msg;
    }
  }
}

await log.setup({
  handlers: {
    console: new Handler("DEBUG"),
  },
  loggers: {
    default: {
      level: args.debug ? "DEBUG" : "INFO",
      handlers: ["console"],
    },
  },
});
