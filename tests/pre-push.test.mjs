import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

const hook = new URL("../.githooks/pre-push", import.meta.url).pathname;
const zero = "0000000000000000000000000000000000000000";

// The pre-push hook passes the range of the pushed commits to `make push-check` (S72). The fixture Makefile writes the
// commits of that range, so each case reads the commits that the hook checked.
const pushed = (remoteSha) => {
  const repository = mkdtempSync(join(tmpdir(), "alacritty-pre-push-"));
  try {
    const git = (...args) => execFileSync("git", args, { cwd: repository, encoding: "utf8" }).trim();
    git("init", "-q", "-b", "main");
    writeFileSync(join(repository, "Makefile"), "push-check:\n\t@git rev-list --no-merges $(RANGE) > range.txt\n");
    git("add", "Makefile");
    git("-c", "user.name=test", "-c", "user.email=test@example.com", "commit", "-q", "-m", "first");
    const first = git("rev-parse", "HEAD");
    git("-c", "user.name=test", "-c", "user.email=test@example.com", "commit", "-q", "--allow-empty", "-m", "second");
    const head = git("rev-parse", "HEAD");
    copyFileSync(hook, join(repository, "pre-push"));
    execFileSync("sh", ["pre-push", "origin", "https://example.com/repository.git"], {
      cwd: repository,
      input: `refs/heads/main ${head} refs/heads/main ${remoteSha(first)}\n`,
      encoding: "utf8",
    });
    return { head, checked: readFileSync(join(repository, "range.txt"), "utf8").trim().split("\n") };
  } finally {
    rmSync(repository, { recursive: true, force: true });
  }
};

test("the first push of a repository checks every commit of the pushed head", { timeout: 10000 }, () => {
  const { checked } = pushed(() => zero);
  assert.equal(checked.length, 2, `the first push checked ${checked.length} commits: ${checked}`);
});

test("a push of an existing remote ref checks the commits after the remote head", { timeout: 10000 }, () => {
  const { head, checked } = pushed((first) => first);
  assert.deepEqual(checked, [head]);
});
