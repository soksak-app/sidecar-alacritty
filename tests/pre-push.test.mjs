import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

const hook = new URL("../.githooks/pre-push", import.meta.url).pathname;
const zero = "0000000000000000000000000000000000000000";

// The pre-push hook passes the range of the pushed commits to `make push-check`. The first push of a repository has no
// origin/main, so the range is every commit of the pushed head.
test("the first push of a repository checks every commit of the pushed head", { timeout: 10000 }, () => {
  const repository = mkdtempSync(join(tmpdir(), "alacritty-pre-push-"));
  try {
    const git = (...args) => execFileSync("git", args, { cwd: repository, encoding: "utf8" }).trim();
    git("init", "-q", "-b", "main");
    writeFileSync(join(repository, "Makefile"), "push-check:\n\t@git rev-list --no-merges $(RANGE) > range.txt\n");
    git("add", "Makefile");
    git("-c", "user.name=test", "-c", "user.email=test@example.com", "commit", "-q", "-m", "first");
    git("-c", "user.name=test", "-c", "user.email=test@example.com", "commit", "-q", "--allow-empty", "-m", "second");
    const head = git("rev-parse", "HEAD");
    copyFileSync(hook, join(repository, "pre-push"));
    execFileSync("sh", ["pre-push", "origin", "https://example.com/repository.git"], {
      cwd: repository,
      input: `refs/heads/main ${head} refs/heads/main ${zero}\n`,
      encoding: "utf8",
    });
    const range = execFileSync("cat", ["range.txt"], { cwd: repository, encoding: "utf8" }).trim().split("\n");
    assert.equal(range.length, 2, `the first push checked ${range.length} commits: ${range}`);
  } finally {
    rmSync(repository, { recursive: true, force: true });
  }
});
