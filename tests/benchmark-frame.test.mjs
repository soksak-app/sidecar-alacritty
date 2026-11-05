import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { WORKLOADS, cpuSeconds, parseArguments } from "../scripts/benchmark-frame.mjs";

test("the benchmark reads its arguments and refuses a missing or an unknown value", { timeout: 1000 }, () => {
  const base = ["--service", "/bin/service", "--answer", "leased", "--workload", "statusline"];
  assert.deepEqual(parseArguments(base), { seconds: 15, service: "/bin/service", answer: "leased", workload: "statusline" });
  assert.equal(parseArguments([...base, "--seconds", "3"]).seconds, 3);
  assert.throws(() => parseArguments(["--answer", "leased", "--workload", "idle"]), /--service is required/);
  assert.throws(() => parseArguments(["--service", "s", "--answer", "copied", "--workload", "idle"]), /--answer must be consumed or leased, found copied/);
  assert.throws(() => parseArguments(["--service", "s", "--answer", "leased", "--workload", "busy"]), /--workload must be one of idle, statusline, fullscreen, found busy/);
  assert.throws(() => parseArguments([...base, "--seconds", "0"]), /--seconds must be a positive number, found 0/);
  assert.throws(() => parseArguments([...base, "--rows", "3"]), /unknown argument --rows/);
  assert.throws(() => parseArguments([...base, "--sample"]), /--sample requires a value/);
});

test("the benchmark reads the CPU time forms of ps", { timeout: 1000 }, () => {
  assert.equal(cpuSeconds(" 0:02.50\n"), 2.5);
  assert.equal(cpuSeconds("1:02.50"), 62.5);
  assert.equal(cpuSeconds("1:00:02.00"), 3602);
  assert.equal(cpuSeconds("1-00:00:01"), 86401);
  assert.throws(() => cpuSeconds("soon"), /invalid CPU time "soon"/);
});

test("the idle workload sends no output and the others draw on a clock", { timeout: 1000 }, () => {
  assert.equal(WORKLOADS.idle, null);
  assert.match(WORKLOADS.statusline, /0\.033/);
  assert.match(WORKLOADS.fullscreen, /0\.033/);
});

// A benchmark that a person or a limit stops ends the service and removes its folder, which a later run does not see.
test("a stopped benchmark ends its service and removes its folder", { timeout: 20000 }, async () => {
  const temporary = mkdtempSync(join(tmpdir(), "benchmark-stop-"));
  try {
    const service = join(temporary, "service.sh");
    const pidFile = join(temporary, "service.pid");
    writeFileSync(service, `#!/bin/sh\necho $$ > ${pidFile}\necho started >&2\nexec sleep 30\n`, { mode: 0o755 });
    const script = fileURLToPath(new URL("../scripts/benchmark-frame.mjs", import.meta.url));
    const benchmark = spawn(process.execPath, [script, "--service", service, "--answer", "leased", "--workload", "idle"], {
      env: { ...process.env, TMPDIR: temporary },
      stdio: ["ignore", "ignore", "pipe"],
    });
    const exited = new Promise((resolve) => benchmark.once("exit", (code, signal) => resolve({ code, signal })));
    await new Promise((resolve) => benchmark.stderr.once("data", resolve));
    benchmark.kill("SIGTERM");
    assert.deepEqual(await exited, { code: 1, signal: null });
    const pid = Number(readFileSync(pidFile, "utf8"));
    assert.throws(() => process.kill(pid, 0), /ESRCH/, "the service still runs");
    assert.deepEqual(readdirSync(temporary).filter((name) => name.startsWith("frame-benchmark-")), []);
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});
