# Changelog

- S1-11: The image surface of a session draws on the GPU frame, which redraws only the rows that changed.
- S1-9: A cell of the screen holds its colors as RGB bytes and its grapheme inline, so building the screen of a frame takes no allocation per cell.
- S1-8: A frame whose record the event log does not store is not hashed.
- S1-6: The process group test waits for the exit stage at which macOS refuses signals.
- S1-4: The sidecar is public as `soksak-app/sidecar-alacritty`, released as 0.0.1 and listed in the registry.
- S1-7: A drawn frame sends a `view` event with its cursor, caret and scrollback when they change.
- S1-4-2: A test runs the pre-push hook and checks its pushed range instead of matching its text.
- S1-4-1: The pre-push hook checks every pushed commit on the first push of a repository.
- S1-3: A frame sends no screen message; `screen.read` answers the full screen.
- S1-2: Output is drawn on a frame clock of 33 ms, at once for 200 ms after an input, and not at all without a change.
- S1-1: The repository starts from the tree of the VT sidecar with the GPU frame, the surface pool and the lease answers.
