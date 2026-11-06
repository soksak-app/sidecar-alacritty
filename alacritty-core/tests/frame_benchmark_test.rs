// The CPU time of a strip frame draw at the 1520x1228 size of the specification, for one changed row, every row changed by
// content and a one-line scroll of a full screen. It measures time, so it runs only when named:
// `cargo test --release -p soksak-sidecar-alacritty-core --test frame_benchmark_test -- --ignored --nocapture`.
use soksak_sidecar_alacritty_core::TerminalTheme;
use soksak_sidecar_alacritty_core::platform::darwin::frame::{
    CursorRender, Frame, UnfocusedCursor, metrics,
};
use soksak_sidecar_alacritty_core::protocol::{Cell, Cursor, CursorShape, Screen};

const WIDTH: u32 = 1520;
const HEIGHT: u32 = 1228;
const WARMUP: usize = 20;
const DRAWS: usize = 300;

// The CPU time of the calling thread; a draw is synchronous, so it runs on this thread.
fn thread_cpu_ns() -> u64 {
    let mut time: libc::timespec = unsafe { std::mem::zeroed() };
    let result = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut time) };
    assert_eq!(result, 0, "clock_gettime refused");
    time.tv_sec as u64 * 1_000_000_000 + time.tv_nsec as u64
}

// Line `n`: shell output of mixed letters, digits and punctuation, distinct for each `n`.
fn line(n: usize, cols: usize) -> Vec<Cell> {
    let text = format!(
        "{n:07} drwxr-xr-x  staff  4096 Nov  6 12:{:02} src/platform/darwin/frame.m ",
        n % 60
    );
    let chars: Vec<char> = text.chars().collect();
    (0..cols)
        .map(|col| Cell {
            ch: Some(chars[col % chars.len()].to_string().as_str().into()),
            ..Cell::default()
        })
        .collect()
}

fn screen(cols: usize, rows: usize) -> Screen {
    Screen {
        cols: cols as u16,
        rows: rows as u16,
        cursor: Cursor {
            col: 0,
            row: (rows - 1) as u16,
            shape: CursorShape::Block,
            visible: true,
            blinking: false,
            blink_visible: true,
            focused: true,
            preedit: None,
        },
        scrollback: Default::default(),
        background: "#1e1e1e".to_string(),
        lines: (0..rows).map(|row| line(row, cols)).collect(),
    }
}

#[test]
#[ignore = "measures time; run it by name in the release profile"]
fn the_cpu_time_of_a_strip_frame_draw() {
    let metrics = metrics(13.0, 2.0);
    let cols = (WIDTH / metrics.cell_width as u32) as usize;
    let rows = (HEIGHT / metrics.cell_height as u32) as usize;
    let theme = TerminalTheme::dark();
    let cursor = CursorRender {
        visible: true,
        focused: true,
        blink_visible: true,
        shape: CursorShape::Block,
        unfocused: UnfocusedCursor::Hollow,
    };
    type Change = fn(&mut Screen, usize, usize, usize);
    let workloads: [(&str, Change); 3] = [
        ("one changed row", |screen, draw, cols, rows| {
            screen.lines[rows - 1] = line(1_000_000 + draw, cols);
        }),
        ("every row changed", |screen, draw, cols, rows| {
            for row in 0..rows {
                screen.lines[row] = line(1_000_000 + draw * rows + row, cols);
            }
        }),
        ("one-line scroll", |screen, draw, cols, rows| {
            screen.lines.remove(0);
            screen.lines.push(line(rows + draw, cols));
        }),
    ];
    for (what, change) in workloads {
        let frame = Frame::new_strip(WIDTH, HEIGHT).expect("strip frame");
        let mut screen = screen(cols, rows);
        let mut times = Vec::with_capacity(DRAWS);
        for draw in 0..WARMUP + DRAWS {
            change(&mut screen, draw, cols, rows);
            let start = thread_cpu_ns();
            frame
                .draw_with_theme(&screen, &metrics, cursor, &theme)
                .expect("draw");
            if draw >= WARMUP {
                times.push(thread_cpu_ns() - start);
            }
        }
        times.sort_unstable();
        let mean = times.iter().sum::<u64>() as f64 / times.len() as f64 / 1e6;
        let median = times[times.len() / 2] as f64 / 1e6;
        let p90 = times[times.len() * 9 / 10] as f64 / 1e6;
        println!(
            "{{\"workload\":\"{what}\",\"size\":\"{WIDTH}x{HEIGHT}\",\"grid\":\"{cols}x{rows}\",\"draws\":{DRAWS},\"mean_ms\":{mean:.3},\"median_ms\":{median:.3},\"p90_ms\":{p90:.3}}}"
        );
    }
}
