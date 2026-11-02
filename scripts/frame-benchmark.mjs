#!/usr/bin/env node
// Measures the CPU time and the footprint of a built terminal service while one surface draws a fixed output. The script
// connects as the host does and answers each frame: `consumed` for a region in the copy mode, or `leased` and then
// `released` when the next frame replaces it for a region in the lease mode. It prints one JSON line.
//
//   node scripts/frame-benchmark.mjs --service <binary> --answer consumed|leased --workload idle|statusline|fullscreen
//     [--seconds <n>] [--sample <file>]
import { execFileSync, spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync } from "node:fs";
import net from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const STEP_MS = 10000;
const WIDTH = 1520;
const HEIGHT = 1228;
const SCALE = 2;

export const WORKLOADS = {
  idle: null,
  statusline: "perl -e '$|=1;for(;;){printf \"\\e[20;1H status %08d\",$i++;select(undef,undef,undef,0.033)}'\n",
  fullscreen: "perl -e '$|=1;for(;;){$i++;print \"\\e[H\";for $r(1..20){printf \"\\e[38;5;%dm%s\\n\",16+($i+$r)%200,chr(97+($i+$r)%26)x60}select(undef,undef,undef,0.033)}'\n",
};

/** Reads the arguments of the command line; an unknown or a missing argument is an error. */
export function parseArguments(argv) {
  const options = { seconds: 15 };
  for (let index = 0; index < argv.length; index += 2) {
    const [name, value] = [argv[index], argv[index + 1]];
    if (!["--service", "--answer", "--workload", "--seconds", "--sample"].includes(name)) throw new Error(`unknown argument ${name}`);
    if (value === undefined) throw new Error(`${name} requires a value`);
    options[name.slice(2)] = name === "--seconds" ? Number(value) : value;
  }
  if (!options.service) throw new Error("--service is required");
  if (!["consumed", "leased"].includes(options.answer)) throw new Error(`--answer must be consumed or leased, found ${options.answer}`);
  if (!Object.hasOwn(WORKLOADS, options.workload)) throw new Error(`--workload must be one of ${Object.keys(WORKLOADS).join(", ")}, found ${options.workload}`);
  if (!Number.isFinite(options.seconds) || options.seconds <= 0) throw new Error(`--seconds must be a positive number, found ${options.seconds}`);
  return options;
}

/** The seconds of a `ps -o time=` value such as `1:02.50` or `1-02:03:04`. */
export function cpuSeconds(text) {
  const match = /^(?:(\d+)-)?([\d:.]+)$/.exec(text.trim());
  const parts = match ? match[2].split(":").map(Number) : [NaN];
  if (parts.some(Number.isNaN)) throw new Error(`invalid CPU time ${JSON.stringify(text)}`);
  return Number(match[1] ?? 0) * 86400 + parts.reduce((total, part) => total * 60 + part, 0);
}

const cpuOf = (pid) => cpuSeconds(execFileSync("ps", ["-o", "time=", "-p", String(pid)], { encoding: "utf8" }));

function footprintMegabytes(pid) {
  const output = execFileSync("footprint", ["-p", String(pid)], { encoding: "utf8" });
  const match = /Footprint:\s*([\d.]+) (KB|MB|GB)/.exec(output);
  if (!match) throw new Error(`footprint printed no total for ${pid}`);
  return Number(match[1]) * { KB: 1 / 1024, MB: 1, GB: 1024 }[match[2]];
}

const within = (promise, what) => {
  let timer;
  const limit = new Promise((_, reject) => { timer = setTimeout(() => reject(new Error(`${what} did not happen within ${STEP_MS} ms`)), STEP_MS); });
  return Promise.race([promise, limit]).finally(() => clearTimeout(timer));
};

async function main(options) {
  const folder = realpathSync(mkdtempSync(join(tmpdir(), "frame-benchmark-")));
  mkdirSync(join(folder, "service"));
  mkdirSync(join(folder, "logs"));
  const service = spawn(options.service, ["--service-dir", join(folder, "service"), "--log-dir", join(folder, "logs")], { stdio: ["ignore", "pipe", "inherit"] });
  // A stop request ends the service and removes the folder, which the end of main does not reach.
  for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) {
    process.once(signal, () => {
      service.once("exit", () => {
        rmSync(folder, { recursive: true, force: true });
        process.exit(1);
      });
      service.kill("SIGTERM");
    });
  }
  try {
    const failed = new Promise((_, reject) => {
      service.once("error", reject);
      service.once("exit", (code, signal) => reject(new Error(`the service ended with ${code ?? signal}`)));
    });
    await within(Promise.race([failed, new Promise((resolve) => service.stdout.once("data", resolve))]), "the ready line of the service");
    const endpoint = JSON.parse(readFileSync(join(folder, "service", "endpoint.json"), "utf8"));
    const socket = net.createConnection(endpoint.socket);
    await within(new Promise((resolve, reject) => { socket.once("connect", resolve); socket.once("error", reject); }), "the connection");
    const surface = "benchmark";
    const send = (body) => socket.write(`${JSON.stringify({ surface, root: folder, body })}\n`);
    let frames = 0;
    let shown = null;
    let firstFrame;
    const drawn = new Promise((resolve) => { firstFrame = resolve; });
    let buffered = "";
    socket.setEncoding("utf8");
    socket.on("data", (chunk) => {
      buffered += chunk;
      for (let newline = buffered.indexOf("\n"); newline >= 0; newline = buffered.indexOf("\n")) {
        const message = JSON.parse(buffered.slice(0, newline));
        buffered = buffered.slice(newline + 1);
        if (message.body?.error) throw new Error(`the service reported ${JSON.stringify(message.body)}`);
        const image = message.body?.image;
        if (message.surface !== surface || !image?.token) continue;
        frames += 1;
        firstFrame();
        const frame = { name: image.name, generation: image.generation, raster: image.raster, sequence: image.sequence };
        send({ image: { [options.answer]: frame } });
        if (options.answer === "leased") {
          if (shown) send({ image: { released: shown } });
          shown = frame;
        }
      }
    });
    socket.write(`${JSON.stringify({ operation: "hello", protocol: 1, token: endpoint.token, client: "frame-benchmark" })}\n`);
    send({ operation: "open", image: "view", shell: "/bin/sh" });
    send({ image: { configure: { name: "view", generation: 1, raster: 1, width: WIDTH, height: HEIGHT, scale: SCALE } } });
    await within(Promise.race([failed, drawn]), "the first frame");
    const command = WORKLOADS[options.workload];
    const input = (text) => send({ operation: "input", actions: [{ type: "insert", text, replacementRange: null, attributed: false }],
      preedit: { text: "", markedRange: null, selectedRange: null, replacementRange: null, attributed: false } });
    if (command) input(command);
    await new Promise((resolve) => setTimeout(resolve, 3000));
    const [startCpu, startFrames, startedAt] = [cpuOf(service.pid), frames, Date.now()];
    await new Promise((resolve) => setTimeout(resolve, options.seconds * 1000));
    const [cpu, count, elapsed] = [cpuOf(service.pid) - startCpu, frames - startFrames, (Date.now() - startedAt) / 1000];
    const footprint = footprintMegabytes(service.pid);
    // The sample runs after the footprint, which the attachment of the sampler enlarges. It runs beside the answers of
    // this process: a service without an answer draws no frame.
    if (options.sample) {
      const sampler = spawn("sample", [String(service.pid), "5", "-file", options.sample], { stdio: "ignore" });
      await new Promise((resolve, reject) => {
        sampler.once("error", reject);
        sampler.once("exit", (code) => (code === 0 ? resolve() : reject(new Error(`sample ended with ${code}`))));
      });
    }
    process.stdout.write(`${JSON.stringify({
      workload: options.workload, answer: options.answer, seconds: Number(elapsed.toFixed(1)), frames: count,
      cpu_percent: Number((cpu / elapsed * 100).toFixed(1)),
      cpu_ms_per_frame: count > 0 ? Number((cpu * 1000 / count).toFixed(2)) : null,
      footprint_mb: Number(footprint.toFixed(1)),
    })}\n`);
    if (command && count === 0) throw new Error(`the workload ${options.workload} drew no frame`);
    if (command) input("\x03");
    send({ operation: "close" });
    socket.destroy();
  } finally {
    const ended = new Promise((resolve) => service.once("exit", resolve));
    service.kill("SIGTERM");
    await ended;
    rmSync(folder, { recursive: true, force: true });
  }
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  main(parseArguments(process.argv.slice(2))).catch((error) => {
    process.stderr.write(`frame-benchmark: ${error.message}\n`);
    process.exit(1);
  });
}
