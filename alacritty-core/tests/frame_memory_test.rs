use soksak_sidecar_alacritty_core::TerminalTheme;
use soksak_sidecar_alacritty_core::platform::darwin::frame::{
    CursorRender, Frame, UnfocusedCursor, metrics,
};
use soksak_sidecar_alacritty_core::protocol::{Cell, Cursor, CursorShape, Screen};

const WIDTH: u32 = 1520;
const HEIGHT: u32 = 1228;
// The pixels of the target, strip and overlay surfaces, plus 30 MB for CoreGraphics and CoreText. A GPU pass makes the
// driver keep at least 70 MB past this limit (S1-13).
const GROWTH_LIMIT: u64 = 3 * (WIDTH as u64) * (HEIGHT as u64) * 4 + 30_000_000;

// The physical footprint of this process, which counts GPU memory that malloc and vmmap sizes do not show.
fn footprint() -> u64 {
    let mut info: libc::rusage_info_v4 = unsafe { std::mem::zeroed() };
    let result = unsafe {
        libc::proc_pid_rusage(
            libc::getpid(),
            libc::RUSAGE_INFO_V4,
            &mut info as *mut libc::rusage_info_v4 as *mut libc::rusage_info_t,
        )
    };
    assert_eq!(result, 0, "proc_pid_rusage refused");
    info.ri_phys_footprint
}

fn screen() -> Screen {
    let cols = 90;
    let rows = 40;
    Screen {
        cols,
        rows,
        cursor: Cursor {
            col: 1,
            row: 0,
            shape: CursorShape::Block,
            visible: true,
            blinking: false,
            blink_visible: true,
            focused: true,
            preedit: None,
        },
        scrollback: Default::default(),
        background: "#1e1e1e".to_string(),
        lines: (0..rows)
            .map(|_| {
                (0..cols)
                    .map(|_| Cell {
                        ch: Some("x".into()),
                        ..Cell::default()
                    })
                    .collect()
            })
            .collect(),
    }
}

#[test]
fn drawing_frames_does_not_grow_the_footprint_past_the_surfaces_it_holds() {
    let metrics = metrics(13.0, 2.0);
    let frame = Frame::new_strip(WIDTH, HEIGHT).expect("strip frame");
    let theme = TerminalTheme::dark();
    let cursor = CursorRender {
        visible: true,
        focused: true,
        blink_visible: true,
        shape: CursorShape::Block,
        unfocused: UnfocusedCursor::Hollow,
    };
    let mut screen = screen();
    let before = footprint();
    for draw in 0..400usize {
        let digit = char::from_digit((draw % 10) as u32, 10).expect("digit");
        screen.lines[39][0].ch = Some(digit.to_string().as_str().into());
        frame
            .draw_with_theme(&screen, &metrics, cursor, &theme)
            .expect("draw");
    }
    let grown = footprint().saturating_sub(before);
    println!("the draws grew the footprint by {} MB", grown / 1_000_000);
    assert!(
        grown <= GROWTH_LIMIT,
        "the draws grew the footprint by {grown} bytes, past the limit of {GROWTH_LIMIT} bytes"
    );
}
