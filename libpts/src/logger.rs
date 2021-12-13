use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use futures_lite::{pin, Stream, StreamExt};

use termion::{color, style};

use crate::log::{Event, EventKind};

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn color(name: &str) -> impl color::Color {
    let hash = calculate_hash(&name);
    let range = 17..230;

    color::AnsiValue(((hash % (range.end - range.start)) + range.start) as u8)
}

fn kind_name(kind: &EventKind) -> &'static str {
    match *kind {
        EventKind::EnterStep => "Enter Step",
        EventKind::ExitStep => "Exit Step",
        EventKind::Send => "Send",
        EventKind::Receive => "Receive",
        EventKind::Assign => "Assign",
        EventKind::Log => "Log",
        EventKind::Verdict => "Verdict",
        EventKind::FinalVerdict => "Verdict",
        EventKind::TestStart => "Test Start",
        EventKind::TestEnd => "Test End",
        EventKind::MatchFailed => "Match",
        EventKind::Timer(_) => "Timer",
        EventKind::Error => "Error",
        EventKind::Ignored => "Ignored",
    }
}

fn kind_color(kind: &EventKind) -> (&'static dyn color::Color, &'static dyn color::Color) {
    match *kind {
        EventKind::EnterStep => (&color::White, &color::Green),
        EventKind::ExitStep => (&color::White, &color::LightRed),
        EventKind::Send => (&color::LightWhite, &color::Cyan),
        EventKind::Receive => (&color::LightWhite, &color::Magenta),
        EventKind::Assign => (&color::LightWhite, &color::Black),
        EventKind::Log => (&color::LightBlack, &color::LightWhite),
        EventKind::Verdict => (&color::LightBlack, &color::Green),
        EventKind::FinalVerdict => (&color::LightBlack, &color::Green),
        EventKind::TestStart => (&color::LightBlack, &color::LightWhite),
        EventKind::TestEnd => (&color::LightBlack, &color::LightWhite),
        EventKind::MatchFailed => (&color::LightWhite, &color::Yellow),
        EventKind::Timer(_) => (&color::Cyan, &color::LightWhite),
        EventKind::Error => (&color::LightWhite, &color::Red),
        EventKind::Ignored => (&color::LightBlack, &color::LightWhite),
    }
}

fn print_header(time: Option<u32>, step: &str, kind: &EventKind, include_kind_name: bool) {
    print!("{}", style::Faint);

    if let Some(time) = time {
        print!("{:06}ms", time)
    } else {
        print!("{:8}", "");
    }

    print!("{} {}", style::Reset, color::Fg(color(step)));

    print!("{:>20}", &step[step.len().saturating_sub(20)..]);
    print!("{} ", style::Reset);

    let (bg, fg) = kind_color(kind);
    print!(
        "{}{}{} {:^10} {} ",
        style::Bold,
        color::Bg(bg),
        color::Fg(fg),
        if include_kind_name {
            kind_name(kind)
        } else {
            ""
        },
        style::Reset
    );
}

fn print_multiline(kind: &EventKind, data: &str) {
    for (index, line) in data.split('\n').enumerate() {
        if index != 0 {
            println!();
            print_header(None, "", kind, false);
        }
        print!("{}", line);
    }
}

pub async fn print<E>(events: impl Stream<Item = Result<Event, E>>) -> Result<Option<String>, E> {
    let mut stack: Vec<String> = Vec::new();
    pin!(events);
    let events = events.try_fold(None, |result, event| {
        let step = stack.last().map(|last| last as &str).unwrap_or("");

        print_header(event.time, step, &event.kind, true);

        match event.kind {
            EventKind::Timer(event) => print!("{:?} ", event),
            _ => {}
        }

        print_multiline(&event.kind, &*event.name);

        match event.kind {
            EventKind::Assign => print!(" :="),
            EventKind::FinalVerdict => print!(" (final)"),
            _ => {}
        }

        if let Some(values) = event.values {
            if event.kind == EventKind::EnterStep {
                print!("(");
            } else {
                print!(" ");
            }
            for (index, argument) in values.iter().enumerate() {
                if index != 0 {
                    print!(", ");
                }
                print_multiline(&event.kind, &*format!("{}", argument));
            }
            if event.kind == EventKind::EnterStep {
                print!(")");
            }
        }

        println!();

        Ok(match event.kind {
            EventKind::FinalVerdict => Some(event.name),
            EventKind::EnterStep => {
                stack.push(event.name);
                result
            }
            EventKind::ExitStep => {
                let pop = stack.pop();
                assert_eq!(pop, Some(event.name));
                result
            }
            _ => result,
        })
    });
    events.await
}
