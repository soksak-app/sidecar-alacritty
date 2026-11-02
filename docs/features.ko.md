# Features

[English](features.md)

- [~] S1 — P1: 프레임을 GPU로 그리고, core lease 방식(F386)으로 보이며, 요청할 때만 screen을 보내는 터미널 sidecar. copy 방식 기준선인 VT sidecar와 나란히 둔다. 기대: 아래 하위 항목. 진행 중: 다음은 S1-2이고 S1-1은 완료했다.
  - [o] S1-1 — P1: 저장소는 GPU frame, 행 clip, surface pool, `leased`와 `released` 응답을 담은 VT sidecar 트리에서 시작한다([GPU frame](spec/alacritty-core.ko.md#gpu-frame), [lease 응답](spec/alacritty-core.ko.md#lease-응답)). 이름은 `@soksak/sidecar-alacritty`, `soksak-alacritty`, `sidecar.alacritty.*`이고 버전은 0.0.1이다. 결과: `make test`가 sidecar를 build하고, Rust test 509개, Node test, 추적된 source를 읽는 event catalog 검사를 통과한다.
  - [ ] S1-2 — P1: service는 보이는 surface를 33 ms 공용 frame clock으로 그리고, 입력 뒤 200 ms 동안은 바로 그리며, 바뀐 것이 없으면 frame을 그리지 않는다.
  - [ ] S1-3 — P1: service는 screen 메시지를 보내지 않고 `screen.read`에 답한다(core F391의 둘째 screen 형식).
  - [ ] S1-4 — P1: 저장소를 `soksak-app/sidecar-alacritty`로 공개하고, core workspace registry에 선언하고, 0.0.1을 release하고 registry에 올린다.
  - [ ] S1-6 — P1: `alacritty-core/tests/pty_lifecycle.rs`의 `a_process_group_whose_members_are_exiting_is_already_terminated`가 네 번 중 한 번 `exiting: true`인 member에 대해 `left: Ok(Sent)`, `right: Ok(Ending)`으로 실패했다. 입력: test는 shell이 exit 중이 될 때까지 기다리고 `kill(-group, SIGKILL)`이 `EPERM`으로 실패하기를 기대한다. 동작: macOS는 exit의 더 이른 단계에서는 signal을 받아들인다. 기대: test가 signal이 거절되는 단계에 이르거나, `kill_process_group`의 계약이 두 단계의 결과를 모두 정한다.
  - [!] S1-5 — P2: 같은 창과 같은 출력으로 frame의 CPU 시간을 VT sidecar와 비교해 잰다. Held: 개발 뒤 측정. Cause: 측정은 sidecar와 plugin `term`의 개발 뒤에 한다. Retry when: S1-4와 plugin `term`의 release가 끝난다.
