use soksak_sidecar_alacritty_core::TerminalTheme;
use soksak_sidecar_alacritty_core::platform::darwin::frame::{
    CursorRender, Frame, Metrics, UnfocusedCursor, metrics,
};
use soksak_sidecar_alacritty_core::protocol::{Cell, Cursor, CursorShape, Screen};

const WIDTH: u32 = 1520;
const HEIGHT: u32 = 1228;
const COLS: usize = 90;
const ROWS: usize = 40;

// The pixels of the target and overlay surfaces, of the strip store, which keeps two strips per row that the frame shows,
// and of the margin below the rows, plus 30 MB for CoreGraphics and CoreText. A GPU pass makes the driver keep at least
// 70 MB past this limit (S1-13).
fn growth_limit(metrics: &Metrics) -> u64 {
    let cell_height = metrics.cell_height as u64;
    let shown_rows = (ROWS as u64).min((HEIGHT as u64).div_ceil(cell_height));
    let margin_rows = (HEIGHT as u64).saturating_sub(ROWS as u64 * cell_height);
    let pixel_row = WIDTH as u64 * 4;
    2 * HEIGHT as u64 * pixel_row
        + 2 * shown_rows * cell_height * pixel_row
        + margin_rows * pixel_row
        + 30_000_000
}

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

// The bytes that the allocations of every malloc zone of this process hold. A draw that keeps memory of its own, such as
// objects left in an autorelease pool, adds to it. Unlike the footprint, it does not move when the allocator keeps or
// returns free pages: over 1000 draws that scroll after the store is full the footprint moved by up to 540672 bytes in
// either direction while this value moved by at most 224 bytes (S110).
fn bytes_in_use() -> u64 {
    let mut stats: libc::malloc_statistics_t = unsafe { std::mem::zeroed() };
    unsafe { libc::malloc_zone_statistics(std::ptr::null_mut(), &mut stats) };
    stats.size_in_use as u64
}

fn screen() -> Screen {
    let cols = COLS;
    let rows = ROWS;
    Screen {
        cols: cols as u16,
        rows: rows as u16,
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

fn cursor() -> CursorRender {
    CursorRender {
        visible: true,
        focused: true,
        blink_visible: true,
        shape: CursorShape::Block,
        unfocused: UnfocusedCursor::Hollow,
    }
}

// The draws after the store holds the strips of the workload, the number of draws that the test makes, and the step of
// the footprint samples (S110).
const SETTLED: usize = 200;
const DRAWS: usize = 1200;
const SAMPLE_STEP: usize = 100;

// The footprint and the bytes in use count the whole process, so one test draws at a time when cargo test runs both in
// one process (S110).
static ALONE: std::sync::Mutex<()> = std::sync::Mutex::new(());

// The least-squares slope of the samples, in bytes per draw.
fn slope(samples: &[(f64, f64)]) -> f64 {
    let count = samples.len() as f64;
    let mean_x = samples.iter().map(|(x, _)| x).sum::<f64>() / count;
    let mean_y = samples.iter().map(|(_, y)| y).sum::<f64>() / count;
    let covariance: f64 = samples
        .iter()
        .map(|(x, y)| (x - mean_x) * (y - mean_y))
        .sum();
    let variance: f64 = samples
        .iter()
        .map(|(x, _)| (x - mean_x) * (x - mean_x))
        .sum();
    covariance / variance
}

// Draws 1200 frames, each after a change of the screen, and bounds three things.
//
// The growth of the footprint, which counts the surfaces, stays within the limit.
//
// Over the second 200 draws, after the store holds the strips of the workload, the bytes in use of the malloc zones grow
// by at most one strip, so a draw keeps no allocation of its own (S110).
//
// The footprint also counts what malloc does not hold: IOSurfaces, the backing of CGImages, vm_allocate pages and the
// caches of CoreText. Its least-squares slope over the 11 samples from draw 200 to draw 1200, one every 100 draws,
// stays at or below one strip per 200 draws. The footprint moves in pages as the allocator keeps or returns free pages:
// in 60 runs of 1200 scrolls its samples from draw 200 spread over a range of up to 540672 bytes, in steps both ways,
// and their slopes lay between -127795 and 50350 bytes per 200 draws (S110). For samples evenly spaced, values anywhere
// within a range R give a slope of at most R times the sum of the positive deviations of x over the sum of their
// squares: 15 / 110 per step for 11 samples, 0.27 R per 200 draws, so the measured range moves the slope by at most
// 147456 bytes, below one strip of 188480. Five samples over 400 draws give 3 / 10 per step, 0.6 R per 200 draws, which
// that range passes. A draw that keeps one strip per 200 draws outside malloc adds 942400 bytes over the 1000 draws,
// past the range of the steps (S110).
fn assert_growth(what: &str, change: fn(&mut Screen, usize)) {
    let _alone = ALONE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let metrics = metrics(13.0, 2.0);
    let frame = Frame::new_strip(WIDTH, HEIGHT).expect("strip frame");
    let theme = TerminalTheme::dark();
    let mut screen = screen();
    let before = footprint();
    let mut settled = 0;
    let mut steady = 0;
    let mut samples = Vec::new();
    for draw in 0..DRAWS {
        change(&mut screen, draw);
        frame
            .draw_with_theme(&screen, &metrics, cursor(), &theme)
            .expect("draw");
        let drawn = draw + 1;
        if drawn == SETTLED {
            settled = bytes_in_use();
        }
        if drawn == 2 * SETTLED {
            steady = bytes_in_use().saturating_sub(settled);
        }
        if drawn >= SETTLED && drawn % SAMPLE_STEP == 0 {
            samples.push((drawn as f64, footprint() as f64));
        }
    }
    let grown = footprint().saturating_sub(before);
    let limit = growth_limit(&metrics);
    let strip = metrics.cell_height as u64 * WIDTH as u64 * 4;
    let trend = slope(&samples) * SETTLED as f64;
    println!(
        "{what}: the draws grew the footprint by {grown} bytes against the limit of {limit}, the second 200 kept {steady} bytes in use and the footprint rose by {trend:.0} bytes per 200 draws, each against one strip of {strip}"
    );
    let mut failures = Vec::new();
    if grown > limit {
        failures.push(format!(
            "the draws grew the footprint by {grown} bytes, past the limit of {limit} bytes"
        ));
    }
    if steady > strip {
        failures.push(format!(
            "the second 200 draws kept {steady} bytes in use, past one strip of {strip} bytes"
        ));
    }
    if trend > strip as f64 {
        failures.push(format!(
            "the footprint rose by {trend:.0} bytes per 200 draws from draw 200 to draw 1200, past one strip of {strip} bytes"
        ));
    }
    assert!(failures.is_empty(), "{what}: {}", failures.join("; "));
}

#[test]
fn drawing_frames_does_not_grow_the_footprint_past_the_surfaces_it_holds() {
    assert_growth("one changed row", |screen, draw| {
        let digit = char::from_digit((draw % 10) as u32, 10).expect("digit");
        screen.lines[39][0].ch = Some(digit.to_string().as_str().into());
    });
}

// Each draw scrolls a new line into the screen, so the strip store fills and drops its oldest strips.
#[test]
fn scrolling_frames_does_not_grow_the_footprint_past_the_strip_store() {
    assert_growth("one-line scrolls", |screen, draw| {
        screen.lines.remove(0);
        let text = format!("{draw:06}");
        let line = (0..COLS)
            .map(|col| Cell {
                ch: Some(text[col % text.len()..col % text.len() + 1].into()),
                ..Cell::default()
            })
            .collect();
        screen.lines.push(line);
    });
}
