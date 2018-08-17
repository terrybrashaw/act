#![feature(duration_as_u128)]

use std::io::{stdout, Write};
use std::thread;
use std::time::{Duration, Instant};

use clap::{App, crate_authors, crate_description, crate_name, crate_version};

use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

fn duration_from_string(string: &str) -> Duration {
    let mut numbers = String::new();
    let mut seconds = 0;
    for character in string.chars() {
        match character {
            's' => {
                seconds += u64::from_str_radix(&numbers, 10).unwrap();
                numbers.clear();
            }
            'm' => {
                seconds += u64::from_str_radix(&numbers, 10).unwrap() * 60;
                numbers.clear();
            }
            'h' => {
                seconds += u64::from_str_radix(&numbers, 10).unwrap() * 60 * 60;
                numbers.clear();
            }
            'd' => {
                seconds += u64::from_str_radix(&numbers, 10).unwrap() * 60 * 60 * 24;
                numbers.clear();
            }
            char if char.is_digit(10) => numbers.push(char),
            ' ' => {}
            char => unimplemented!(),
        }
    }
    Duration::from_secs(seconds)
}

fn string_from_duration(duration: Duration) -> String {
    let seconds = duration.as_secs() % 60;
    let minutes = duration.as_secs() / 60 % 60;
    let hours = duration.as_secs() / 60 / 60 % 24;
    let days = duration.as_secs() / 60 / 60 / 24;

    if days > 0 {
        format!("{}d{}h{}m{}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h{}m{}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m{}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

fn run(countdown: Duration) -> bool {
    // To ensure the console is returned back into its normal state after we're done, we
    // instantiate this `ConsoleReset` object which resets the console when dropped. This way, the
    // console will always be reset, even if we forget to do it manually or we panic while
    // rendering.
    let _console_reset = ConsoleReset;

    let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());
    let mut stdin = termion::async_stdin().keys();

    let mut elapsed = Duration::default();
    let mut dt = Instant::now();

    let mut paused = false;
    loop {
        if !paused {
            elapsed += dt.elapsed();
        }
        dt = Instant::now();

        if countdown >= elapsed {
            let remaining = string_from_duration(countdown - elapsed);
            let (window_width, window_height) = termion::terminal_size().unwrap();
            if paused {
                write!(stdout, "{}", termion::color::Fg(termion::color::Green));
            }
            write!(
                stdout,
                "{}{}{}{}",
                termion::clear::All,
                termion::cursor::Goto(
                    window_width / 2 - remaining.len() as u16 / 2,
                    window_height / 2
                ),
                remaining,
                termion::cursor::Hide
            );
            write!(stdout, "{}", termion::color::Fg(termion::color::Reset));
            stdout.flush().unwrap();
        } else {
            return true;
        }

        while let Some(key) = stdin.next() {
            match key.unwrap() {
                termion::event::Key::Ctrl('c') => return false,
                termion::event::Key::Esc => return false,
                termion::event::Key::Char(' ') => paused = !paused,
                _ => {}
            }
        }

        thread::sleep(Duration::from_millis(16));
        // thread::sleep(Duration::from_millis(1000 - start_instant.elapsed().subsec_millis() as u64));
    }
}

struct ConsoleReset;
impl Drop for ConsoleReset {
    fn drop(&mut self) {
        print!("{}", termion::cursor::Show);
        print!("{}", termion::color::Fg(termion::color::Reset));
        print!("{}", termion::color::Bg(termion::color::Reset));
    }
}

fn cli() -> App<'static, 'static> {
    clap::App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(clap::AppSettings::UnifiedHelpMessage)
        .max_term_width(80)
        .arg(clap::Arg::with_name("duration")
             .help("Some amount of time to countdown from, specified as a combination of common units of time: '1d', '1h', '1m', '1s'.\n\nExamples: '3d4h', '1m30s', '10d3h21m10s'.")
             .required(true)
        )
}

fn main() {
    let args = cli().get_matches();
    
    // Get the countdown duration, passed in as an app argument. Then, add 1 second so that the
    // amount of time set to countdown from is what's actually shown when the app starts.
    let countdown =
        duration_from_string(args.value_of("duration").unwrap()) + Duration::from_secs(1);

    let finished = run(countdown);
    if finished {
        print!("{}", BELL);
    }
}

/// Escape sequence for `BEL`.
///
/// Typically causes the terminal emulator to play a sound and/or flash
/// the window. On i3, it'll even mark the workspace playing the `BEL` as urgent.
const BELL: &str = "\x07";

#[test]
fn parse_seconds() {
    assert_eq!(duration_from_string("30s"), Duration::from_secs(30));
}

#[test]
fn parse_minutes() {
    assert_eq!(duration_from_string("35m"), Duration::from_secs(35 * 60));
}

#[test]
fn parse_hours() {
    assert_eq!(duration_from_string("3h"), Duration::from_secs(3 * 60 * 60));
}

#[test]
fn parse_seconds_and_minutes_and_hours() {
    assert_eq!(
        duration_from_string("25m100s"),
        Duration::from_secs(25 * 60 + 100)
    );
    assert_eq!(
        duration_from_string("1h1h1h"),
        Duration::from_secs(3 * 60 * 60)
    );
}
