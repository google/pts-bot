use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{self, Write};

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
        EventKind::ManMachineInterface => "MMI",
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
        EventKind::ManMachineInterface => (&color::LightWhite, &color::Yellow),
        EventKind::Ignored => (&color::LightBlack, &color::LightWhite),
    }
}

fn print_header(
    to: &mut impl Write,
    time: Option<u32>,
    step: &str,
    kind: &EventKind,
    include_kind_name: bool,
) -> io::Result<()> {
    write!(to, "{}", style::Faint)?;

    if let Some(time) = time {
        write!(to, "{:06}ms", time)?;
    } else {
        write!(to, "{:8}", "")?;
    }

    write!(to, "{} {}", style::Reset, color::Fg(color(step)))?;

    write!(to, "{:>20}", &step[step.len().saturating_sub(20)..])?;
    write!(to, "{} ", style::Reset)?;

    let (bg, fg) = kind_color(kind);
    write!(
        to,
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
    )
}

fn print_multiline(to: &mut impl Write, kind: &EventKind, data: &str) -> io::Result<()> {
    for (index, line) in data.split('\n').enumerate() {
        if index != 0 {
            writeln!(to)?;
            print_header(to, None, "", kind, false)?;
        }
        if *kind == EventKind::ManMachineInterface {
            write!(to, "{}{}", style::Bold, color::Fg(color::LightYellow))?;
        }
        write!(to, "{}", line)?;
    }
    Ok(())
}

pub fn print(to: &mut impl Write, event: &Event, stack: &[String]) -> io::Result<()> {
    let step = stack.last().map(|last| last as &str).unwrap_or("");

    print_header(to, event.time, step, &event.kind, true)?;

    if let EventKind::Timer(event) = event.kind {
        write!(to, "{:?} ", event)?;
    }

    print_multiline(to, &event.kind, &event.name)?;

    match event.kind {
        EventKind::Assign => write!(to, " :=")?,
        EventKind::FinalVerdict => write!(to, " (final)")?,
        _ => {}
    }

    if let Some(values) = &event.values {
        if event.kind == EventKind::EnterStep {
            write!(to, "(")?;
        } else {
            write!(to, " ")?;
        }
        for (index, argument) in values.iter().enumerate() {
            if index != 0 {
                write!(to, ", ")?;
            }
            print_multiline(to, &event.kind, &format!("{}", argument))?;
        }
        if event.kind == EventKind::EnterStep {
            write!(to, ")")?;
        }
    }

    writeln!(to)
}
