# Changelog

- S1-3: frame은 screen 메시지를 보내지 않고, `screen.read`가 화면 전체로 답한다.
- S1-2: 출력은 33 ms frame clock으로 그리고, 입력 뒤 200 ms 동안은 바로 그리며, 바뀐 것이 없으면 그리지 않는다.
- S1-1: 저장소는 GPU frame, surface pool, lease 응답을 담은 VT sidecar 트리에서 시작한다.
