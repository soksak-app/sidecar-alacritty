# Features

[한국어](features.ko.md)

- [~] S1 — P1: A terminal sidecar that draws its frames on the GPU, presents them in the lease mode of core (F386) and sends screens only on request, beside the VT sidecar that stays the copy-mode baseline. Expected: the children below. Active: S1-2 is next; S1-1 is complete.
  - [o] S1-1 — P1: The repository starts from the tree of the VT sidecar with the GPU frame, the row clip, the surface pool and the `leased` and `released` answers ([GPU frame](spec/alacritty-core.md#gpu-frame), [lease answers](spec/alacritty-core.md#lease-answers)), under the names `@soksak/sidecar-alacritty`, `soksak-alacritty` and `sidecar.alacritty.*`, at version 0.0.1. Result: `make test` builds the sidecar and passes its 509 Rust tests, its Node tests and its event catalog check, which reads the tracked sources.
  - [ ] S1-2 — P1: The service draws the shown surfaces on one frame clock of 33 ms, draws at once for 200 ms after an input, and draws no frame without a change.
  - [ ] S1-3 — P1: The service sends no screen message and answers `screen.read` (the second screen form of core F391).
  - [ ] S1-4 — P1: Publish the repository as `soksak-app/sidecar-alacritty`, declare it in the workspace registry of core, release 0.0.1 and list it in the registry.
  - [ ] S1-6 — P1: `a_process_group_whose_members_are_exiting_is_already_terminated` of `alacritty-core/tests/pty_lifecycle.rs` failed once of four runs with `left: Ok(Sent)`, `right: Ok(Ending)` for a member with `exiting: true`. Input: the test waits until the shell is exiting and expects `kill(-group, SIGKILL)` to fail with `EPERM`. Behaviour: macOS still accepts the signal at an earlier stage of the exit. Expected: the test reaches the stage at which the signal is refused, or the contract of `kill_process_group` names the result for both stages.
  - [!] S1-5 — P2: Measure the CPU time of a frame against the VT sidecar with the same window and output. Held: measurement after development. Cause: measurement runs after the development of the sidecar and the plugin `term`. Retry when: S1-4 and the release of the plugin `term` are complete.
