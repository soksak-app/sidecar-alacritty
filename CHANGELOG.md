# Changelog

- S1-4: The sidecar is public as `soksak-app/sidecar-alacritty`, released as 0.0.1 and listed in the registry.
- S1-7: A drawn frame sends a `view` event with its cursor, caret and scrollback when they change.
- S1-4-2: A test runs the pre-push hook and checks its pushed range instead of matching its text.
- S1-4-1: The pre-push hook checks every pushed commit on the first push of a repository.
- S1-3: A frame sends no screen message; `screen.read` answers the full screen.
- S1-2: Output is drawn on a frame clock of 33 ms, at once for 200 ms after an input, and not at all without a change.
- S1-1: The repository starts from the tree of the VT sidecar with the GPU frame, the surface pool and the lease answers.
