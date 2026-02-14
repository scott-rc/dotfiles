import {
  assert,
  assertEquals,
  assertMatch,
  assertStringIncludes,
} from "@std/assert";

const decoder = new TextDecoder();

interface RunResult {
  code: number;
  stdout: string;
  stderr: string;
}

async function run(args: string[], stdin?: string): Promise<RunResult> {
  const cmd = new Deno.Command("deno", {
    args: ["run", "-A", "main.ts", ...args],
    cwd: import.meta.dirname!,
    stdout: "piped",
    stderr: "piped",
    stdin: stdin !== undefined ? "piped" : "null",
  });
  const proc = cmd.spawn();

  if (stdin !== undefined) {
    const writer = proc.stdin.getWriter();
    await writer.write(new TextEncoder().encode(stdin));
    await writer.close();
  }

  const { code, stdout, stderr } = await proc.output();
  return {
    code,
    stdout: decoder.decode(stdout),
    stderr: decoder.decode(stderr),
  };
}

// -- help --

Deno.test("--help prints usage and exits 0", async () => {
  const { code, stdout } = await run(["--help"]);
  assertEquals(code, 0);
  assertStringIncludes(stdout, "md — terminal markdown renderer");
  assertStringIncludes(stdout, "--no-pager");
});

// -- file rendering --

Deno.test("file renders to stdout", async () => {
  const { code, stdout } = await run(["--no-color", "--no-pager", "README.md"]);
  assertEquals(code, 0);
  assertStringIncludes(stdout, "# MD");
});

// -- stdin --

Deno.test("stdin via pipe renders to stdout", async () => {
  const { code, stdout } = await run(
    ["--no-color", "--no-pager"],
    "# Hello\n",
  );
  assertEquals(code, 0);
  assertStringIncludes(stdout, "# HELLO");
});

Deno.test("explicit - reads stdin", async () => {
  const { code, stdout } = await run(
    ["--no-color", "--no-pager", "-"],
    "# Hello\n",
  );
  assertEquals(code, 0);
  assertStringIncludes(stdout, "# HELLO");
});

// -- flags --

Deno.test("--no-color disables ANSI escapes", async () => {
  const { code, stdout } = await run([
    "--no-color",
    "--no-pager",
    "README.md",
  ]);
  assertEquals(code, 0);
  assert(stdout.length > 0, "expected non-empty output");
  assertMatch(stdout, /^[^\x1b]*$/, "expected no ANSI escape sequences");
});

Deno.test("--width constrains output width", async () => {
  const input = "A word ".repeat(20).trim() + "\n";
  const { code, stdout } = await run(
    ["--no-color", "--no-pager", "--width", "40"],
    input,
  );
  assertEquals(code, 0);
  for (const line of stdout.split("\n")) {
    assert(
      line.length <= 40,
      `line exceeds 40 chars (${line.length}): ${JSON.stringify(line)}`,
    );
  }
});

// -- edge cases --

Deno.test("empty file renders without error", async () => {
  const tmp = await Deno.makeTempFile({ suffix: ".md" });
  try {
    const { code } = await run(["--no-color", "--no-pager", tmp]);
    assertEquals(code, 0);
  } finally {
    await Deno.remove(tmp);
  }
});

// -- error handling --

Deno.test("nonexistent file prints error and exits with code 1", async () => {
  const { code, stderr } = await run(["nonexistent-file.md"]);
  assertEquals(code, 1);
  assertStringIncludes(stderr, "nonexistent-file.md");
  assertStringIncludes(stderr, "not found");
});

Deno.test("nonexistent directory prints error and exits with code 1", async () => {
  const { code, stderr } = await run(["nonexistent-dir/"]);
  assertEquals(code, 1);
  assertStringIncludes(stderr, "nonexistent-dir/");
  assertStringIncludes(stderr, "not found");
});
