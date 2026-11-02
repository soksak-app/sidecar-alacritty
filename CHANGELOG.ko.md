# Changelog

- S1-4: sidecar는 `soksak-app/sidecar-alacritty`로 공개되고, 0.0.1로 release되고, registry에 등록됐다.
- S1-7: 그린 frame은 커서, caret, scrollback이 바뀌면 그것을 담은 `view` 이벤트를 보낸다.
- S1-4-2: test가 pre-push hook의 text를 맞추지 않고 hook을 실행해 push range를 검사한다.
- S1-4-1: pre-push hook은 저장소의 첫 push에서 push하는 commit을 모두 검사한다.
- S1-3: frame은 screen 메시지를 보내지 않고, `screen.read`가 화면 전체로 답한다.
- S1-2: 출력은 33 ms frame clock으로 그리고, 입력 뒤 200 ms 동안은 바로 그리며, 바뀐 것이 없으면 그리지 않는다.
- S1-1: 저장소는 GPU frame, surface pool, lease 응답을 담은 VT sidecar 트리에서 시작한다.
