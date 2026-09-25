//! Reference tests: how close Erk's rendering is to Chrome's, per page.
//!
//! Chrome's screenshots are captured once and committed
//! (`tests/reference/chrome/`), so this test needs no browser and gives the
//! same score on every machine: Erk's output is deterministic. Erk will not
//! match Chrome pixel for pixel (antialiasing and hinting differ), so each
//! page has a similarity score, and the score may never drop below the one
//! recorded in `tests/reference/expectations.txt`. When it rises, the
//! expectation is raised in the same commit: a ratchet.
//!
//! The score counts only content pixels (pixels that are not the canvas
//! colour in either image); otherwise a page with no text drawn at all would
//! still score in the nineties.
//!
//! Run with `-- --nocapture` for the score table. Diff images go to
//! `target/reference-diff/`.
//!
//! To capture Chrome references (new page, or new Chrome version):
//! `cargo test -p erk-renderer --test chrome_reference -- --ignored capture_chrome_references`
//! Chrome is found through `ERK_CHROME` or its default install path.

use std::path::{Path, PathBuf};
use std::process::Command;

const WIDTH: u16 = 800;
const HEIGHT: u16 = 600;

/// Largest per-channel difference (0-255) at which two pixels still count
/// as the same: absorbs antialiasing, not misplaced glyphs.
const TOLERANCE: u8 = 24;

/// How far below its expectation a score may land before the test fails.
/// Erk is deterministic, so this only absorbs rounding in the printout.
const SLACK: f64 = 0.05;

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn reference_dir() -> PathBuf {
    manifest().join("tests/reference")
}

/// Every reference page: `(name, path)`.
fn pages() -> Vec<(String, PathBuf)> {
    let mut pages = vec![(
        "merhaba".to_owned(),
        manifest().join("../../examples/merhaba.html"),
    )];
    let mut dir: Vec<_> = std::fs::read_dir(reference_dir().join("pages"))
        .expect("tests/reference/pages exists")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "html"))
        .collect();
    dir.sort();
    for path in dir {
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();
        pages.push((name, path));
    }
    pages
}

struct Image {
    width: u32,
    height: u32,
    /// Straight RGBA8.
    pixels: Vec<u8>,
}

fn decode(bytes: &[u8]) -> Image {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().expect("valid PNG");
    let mut buffer = vec![0; reader.output_buffer_size().expect("sized PNG")];
    let info = reader.next_frame(&mut buffer).expect("PNG frame");
    buffer.truncate(info.buffer_size());
    let pixels = match info.color_type {
        png::ColorType::Rgba => buffer,
        png::ColorType::Rgb => buffer
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        other => panic!("unexpected PNG colour type {other:?}"),
    };
    Image {
        width: info.width,
        height: info.height,
        pixels,
    }
}

fn encode(image: &Image) -> Vec<u8> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, image.width, image.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&image.pixels)
        .unwrap();
    out
}

struct Comparison {
    /// Percentage of content pixels that match.
    content_score: f64,
    /// Percentage of all pixels that match.
    overall_score: f64,
    diff: Image,
}

fn most_common_colour(image: &Image) -> [u8; 3] {
    let mut counts = std::collections::HashMap::new();
    for p in image.pixels.as_chunks::<4>().0 {
        *counts.entry([p[0], p[1], p[2]]).or_insert(0u32) += 1;
    }
    counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(colour, _)| colour)
        .unwrap_or([255, 255, 255])
}

fn compare(erk: &Image, chrome: &Image) -> Comparison {
    assert_eq!(
        (erk.width, erk.height),
        (chrome.width, chrome.height),
        "Erk and Chrome images differ in size"
    );
    let canvas = most_common_colour(chrome);
    let channel_diff = |a: &[u8], b: &[u8]| (0..3).map(|i| a[i].abs_diff(b[i])).max().unwrap();

    let (mut matched, mut content, mut content_matched) = (0u64, 0u64, 0u64);
    let mut diff = Vec::with_capacity(erk.pixels.len());
    for (e, c) in erk
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .zip(chrome.pixels.as_chunks::<4>().0)
    {
        let same = channel_diff(e, c) <= TOLERANCE;
        let is_content =
            channel_diff(e, &canvas) > TOLERANCE || channel_diff(c, &canvas) > TOLERANCE;
        matched += u64::from(same);
        if is_content {
            content += 1;
            content_matched += u64::from(same);
        }
        // Red where they disagree, a faint grey copy of Chrome elsewhere.
        if same {
            let grey = 200 + (u16::from(c[0]) + u16::from(c[1]) + u16::from(c[2])) as u8 / 3 / 5;
            diff.extend_from_slice(&[grey, grey, grey, 255]);
        } else {
            diff.extend_from_slice(&[230, 30, 30, 255]);
        }
    }
    let total = erk.pixels.len() as f64 / 4.0;
    Comparison {
        content_score: if content == 0 {
            100.0
        } else {
            100.0 * content_matched as f64 / content as f64
        },
        overall_score: 100.0 * matched as f64 / total,
        diff: Image {
            width: erk.width,
            height: erk.height,
            pixels: diff,
        },
    }
}

fn expectations() -> Vec<(String, f64)> {
    std::fs::read_to_string(reference_dir().join("expectations.txt"))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (name, score) = line.split_once(char::is_whitespace).expect("`name score`");
            (
                name.to_owned(),
                score.trim().parse().expect("numeric score"),
            )
        })
        .collect()
}

#[test]
fn erk_does_not_drift_away_from_chrome() {
    let expected = expectations();
    let out = manifest().join("../../target/reference-diff");
    std::fs::create_dir_all(&out).unwrap();

    let mut report = String::from("page            content  overall  expected\n");
    let mut failures = Vec::new();
    for (name, path) in pages() {
        let chrome_path = reference_dir().join("chrome").join(format!("{name}.png"));
        let Ok(chrome_png) = std::fs::read(&chrome_path) else {
            failures.push(format!(
                "{name}: no Chrome reference; capture it (see module docs)"
            ));
            continue;
        };
        let html = std::fs::read_to_string(&path).unwrap();
        let erk_png = erk_renderer::render_html(&html, WIDTH, HEIGHT).to_png();
        let result = compare(&decode(&erk_png), &decode(&chrome_png));

        std::fs::write(out.join(format!("{name}.erk.png")), &erk_png).unwrap();
        std::fs::write(out.join(format!("{name}.chrome.png")), &chrome_png).unwrap();
        std::fs::write(out.join(format!("{name}.diff.png")), encode(&result.diff)).unwrap();

        let expectation = expected.iter().find(|(n, _)| *n == name).map(|&(_, s)| s);
        report.push_str(&format!(
            "{name:<15} {:>6.2}%  {:>6.2}%  {}\n",
            result.content_score,
            result.overall_score,
            expectation.map_or("-".to_owned(), |s| format!("{s:.2}%"))
        ));
        match expectation {
            None => failures.push(format!(
                "{name}: no expectation; add `{name} {:.2}` to expectations.txt",
                result.content_score
            )),
            Some(s) if result.content_score < s - SLACK => failures.push(format!(
                "{name}: content score {:.2}% fell below the expected {s:.2}%",
                result.content_score
            )),
            Some(s) if result.content_score > s + 0.5 => println!(
                "{name}: improved to {:.2}% (expected {s:.2}%); raise the expectation",
                result.content_score
            ),
            Some(_) => {}
        }
    }
    std::fs::write(out.join("report.txt"), &report).unwrap();
    println!("\n{report}");
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

fn find_chrome() -> PathBuf {
    if let Some(path) = std::env::var_os("ERK_CHROME") {
        return path.into();
    }
    let candidates: &[&str] = if cfg!(windows) {
        &[
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        ]
    } else if cfg!(target_os = "macos") {
        &["/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"]
    } else {
        &[
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
            "/usr/bin/chromium",
        ]
    };
    candidates
        .iter()
        .map(PathBuf::from)
        .find(|path| path.exists())
        .expect("Chrome not found; set ERK_CHROME")
}

fn file_url(path: &Path) -> String {
    let path = path.canonicalize().unwrap();
    let path = path.to_string_lossy().replace('\\', "/");
    let path = path.trim_start_matches("//?/");
    if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    }
}

/// Capture Chrome's rendering of every reference page. Not part of the
/// normal run: it needs Chrome, and references change only when a page is
/// added or Chrome is upgraded.
#[test]
#[ignore]
fn capture_chrome_references() {
    let chrome = find_chrome();
    let work = manifest().join("../../target/chrome-capture");
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).unwrap();
    let chrome_dir = reference_dir().join("chrome");
    std::fs::create_dir_all(&chrome_dir).unwrap();

    // Chrome must draw with the same fonts Erk embeds.
    let fonts = manifest().join("assets/fonts");
    let font_face = format!(
        "<style>@font-face {{ font-family: \"Noto Sans\"; font-weight: 400; src: url(\"{}\"); }}\n\
         @font-face {{ font-family: \"Noto Sans\"; font-weight: 700; src: url(\"{}\"); }}</style>\n",
        file_url(&fonts.join("NotoSans-Regular.ttf")),
        file_url(&fonts.join("NotoSans-Bold.ttf")),
    );

    for (name, path) in pages() {
        let html = std::fs::read_to_string(&path).unwrap();
        let page = work.join(format!("{name}.html"));
        // Inside <head>, after the doctype: anything before `<!DOCTYPE html>`
        // puts Chrome into quirks mode, where the body's first child loses
        // its top margin and every page shifts.
        let at = html
            .find("<head>")
            .map(|i| i + "<head>".len())
            .expect("reference pages have a <head>");
        std::fs::write(&page, format!("{}{font_face}{}", &html[..at], &html[at..])).unwrap();
        let shot = chrome_dir.join(format!("{name}.png"));
        let status = Command::new(&chrome)
            .args([
                "--headless",
                "--disable-gpu",
                "--hide-scrollbars",
                "--force-device-scale-factor=1",
                "--disable-lcd-text",
                "--allow-file-access-from-files",
                "--no-first-run",
                "--no-default-browser-check",
                &format!("--window-size={WIDTH},{HEIGHT}"),
                &format!("--user-data-dir={}", work.join("profile").display()),
                &format!("--screenshot={}", shot.display()),
                &file_url(&page),
            ])
            .status()
            .expect("Chrome runs");
        assert!(status.success(), "Chrome failed on {name}");
        println!("captured {name}");
    }

    // On Windows, `chrome.exe --version` does not print a version: it hands
    // the command line to an already running Chrome, which opens a window in
    // the user's own browser. The version has to come from ERK_CHROME_VERSION
    // there.
    let version = std::env::var("ERK_CHROME_VERSION")
        .ok()
        .or_else(|| {
            if cfg!(windows) {
                return None;
            }
            Command::new(&chrome)
                .arg("--version")
                .output()
                .ok()
                .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
                .filter(|version| !version.is_empty())
        })
        .unwrap_or_else(|| "unknown Chrome version".to_owned());
    std::fs::write(
        chrome_dir.join("VERSION.txt"),
        format!(
            "Captured with: {version} ({})\nWindow {WIDTH}x{HEIGHT}, device scale 1, LCD text off, embedded Noto Sans.\n",
            std::env::consts::OS
        ),
    )
    .unwrap();
}
