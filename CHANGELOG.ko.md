# Changelog

- S1-18: sidecar는 records kit 0.0.6을 쓴다.
- S1-16: service의 시작 기록과 stall dump는 sidecar를 `alacritty`로 적는다.
- S1-15: `make test`는 diagnostics build의 frame test를 모두 실행한다.
- S1-14: `make frame-benchmark`는 빌드한 service의 CPU 시간과 footprint를 잰다.
- S1-13: frame은 GPU pass 없이 그린다. 부하가 걸린 service의 메모리가 같은 CPU 시간으로 196 MB에서 40 MB로 줄었다.
- S1-12: sidecar를 0.0.2로 release하고 registry에 등록했다.
- S1-11: session의 image surface는 바뀐 행만 다시 그리는 GPU frame으로 그린다.
- S1-9: screen의 셀은 색을 RGB byte로, grapheme을 inline으로 가지므로 frame의 screen을 만들 때 셀마다 할당하지 않는다.
- S1-8: event log가 record를 저장하지 않는 frame은 hash하지 않는다.
- S1-6: process group test는 macOS가 signal을 거부하는 exit 단계를 기다린다.
- S1-4: sidecar는 `soksak-app/sidecar-alacritty`로 공개되고, 0.0.1로 release되고, registry에 등록됐다.
- S1-7: 그린 frame은 커서, caret, scrollback이 바뀌면 그것을 담은 `view` 이벤트를 보낸다.
- S1-4-2: test가 pre-push hook의 text를 맞추지 않고 hook을 실행해 push range를 검사한다.
- S1-4-1: pre-push hook은 저장소의 첫 push에서 push하는 commit을 모두 검사한다.
- S1-3: frame은 screen 메시지를 보내지 않고, `screen.read`가 화면 전체로 답한다.
- S1-2: 출력은 33 ms frame clock으로 그리고, 입력 뒤 200 ms 동안은 바로 그리며, 바뀐 것이 없으면 그리지 않는다.
- S1-1: 저장소는 GPU frame, surface pool, lease 응답을 담은 VT sidecar 트리에서 시작한다.
