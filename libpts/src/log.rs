use crate::pts::{LogType, Message};
use crate::ttcn;
use std::iter::Iterator;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TimerEvent {
    Start,
    Stop,
    Cancel,
    Read,
    Timeout,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EventKind {
    EnterStep,
    ExitStep,
    Send,
    Receive,
    Assign,
    Log,
    Verdict,
    FinalVerdict,
    TestStart,
    TestEnd,
    MatchFailed,
    Timer(TimerEvent),
    Error,
    Ignored,
}

#[derive(Debug)]
pub struct Event {
    pub kind: EventKind,
    pub time: Option<u32>,
    pub number: Option<String>,
    pub name: String,
    pub values: Option<Vec<ttcn::TTCNValue>>,
}

fn parse_log_message(logtype: LogType, message: String) -> Event {
    let message = message.trim();

    match logtype {
        LogType::Attach => {
            let split: Vec<&str> = message.split(" ").collect();

            match split[..] {
                [":", number, "Enter", "Test", "Step", step] => Event {
                    kind: EventKind::EnterStep,
                    time: None,
                    number: Some(number.to_owned()),
                    name: step.to_owned(),
                    values: Some(vec![]),
                },
                [":", number, "Enter", "Test", "Step", step, "(", .., ")"] => {
                    let index = split[0..7].iter().map(|s| s.len()).sum::<usize>() + 6;
                    let input = &message[index..message.len() - 1];

                    let (input, values) = ttcn::comma_separated_values(input).unwrap();
                    assert!(input.is_empty());

                    Event {
                        kind: EventKind::EnterStep,
                        time: None,
                        number: Some(number.to_owned()),
                        name: step.to_owned(),
                        values: Some(values),
                    }
                }
                [":", number, "Exit", "", "Test", "Step", "", step] => Event {
                    kind: EventKind::ExitStep,
                    time: None,
                    number: Some(number.to_owned()),
                    name: step.to_owned(),
                    values: None,
                },
                _ => todo!(),
            }
        }
        LogType::SendEvent => {
            if let Some((name, pdu)) = message.split_once("=PDU:") {
                let (input, value) = ttcn::parse(pdu).unwrap();
                assert!(input.is_empty());

                Event {
                    kind: EventKind::Send,
                    time: None,
                    number: None,
                    name: name.to_owned(),
                    values: Some(vec![value]),
                }
            } else {
                let name = message.split_whitespace().collect::<Vec<_>>().join(" ");

                Event {
                    kind: EventKind::Send,
                    time: None,
                    number: None,
                    name,
                    values: None,
                }
            }
        }
        LogType::ReceiveEvent => {
            if let Some((name, pdu)) = message.split_once("=PDU:") {
                let (input, value) = ttcn::parse(pdu).unwrap();
                assert!(input.is_empty());

                Event {
                    kind: EventKind::Receive,
                    time: None,
                    number: None,
                    name: name.to_owned(),
                    values: Some(vec![value]),
                }
            } else {
                let name = message.split_whitespace().collect::<Vec<_>>().join(" ");

                Event {
                    kind: EventKind::Receive,
                    time: None,
                    number: None,
                    name,
                    values: None,
                }
            }
        }
        LogType::Assignment => {
            if let Some((name, value)) = message.split_once(":=") {
                let (input, value) = ttcn::parse(value).unwrap();
                assert!(input.is_empty());

                Event {
                    kind: EventKind::Assign,
                    time: None,
                    number: None,
                    name: name.to_owned(),
                    values: Some(vec![value]),
                }
            } else {
                unreachable!();
            }
        }
        LogType::GeneralText => Event {
            kind: EventKind::Log,
            time: None,
            number: None,
            name: message.to_owned(),
            values: None,
        },
        LogType::FinalVerdict => {
            if let Some(log) = message.strip_prefix("OUTPUT/") {
                Event {
                    kind: EventKind::Log,
                    time: None,
                    number: None,
                    name: log.to_owned(),
                    values: None,
                }
            } else if let Some(verdict) = message.strip_prefix("VERDICT/") {
                Event {
                    kind: EventKind::FinalVerdict,
                    time: None,
                    number: None,
                    name: verdict.to_owned(),
                    values: None,
                }
            } else {
                Event {
                    kind: EventKind::Verdict,
                    time: None,
                    number: None,
                    name: message.to_owned(),
                    values: None,
                }
            }
        }
        LogType::PreliminaryVerdict => Event {
            kind: EventKind::Verdict,
            time: None,
            number: None,
            name: message.to_owned(),
            values: None,
        },
        LogType::StartTestCase => Event {
            kind: EventKind::TestStart,
            time: None,
            number: None,
            name: message.to_owned(),
            values: None,
        },
        LogType::TestCaseEnded => Event {
            kind: EventKind::TestEnd,
            time: None,
            number: None,
            name: message.to_owned(),
            values: None,
        },
        LogType::MatchFailed => Event {
            kind: EventKind::MatchFailed,
            time: None,
            number: None,
            name: message.to_owned(),
            values: None,
        },
        LogType::StartTimer => Event {
            kind: EventKind::Timer(TimerEvent::Start),
            time: None,
            number: None,
            name: message.to_owned(),
            values: None,
        },
        LogType::StopTimer => Event {
            kind: EventKind::Timer(TimerEvent::Stop),
            time: None,
            number: None,
            name: message.to_owned(),
            values: None,
        },
        LogType::CancelTimer => Event {
            kind: EventKind::Timer(TimerEvent::Cancel),
            time: None,
            number: None,
            name: message.to_owned(),
            values: None,
        },
        LogType::ReadTimer => Event {
            kind: EventKind::Timer(TimerEvent::Read),
            time: None,
            number: None,
            name: message.to_owned(),
            values: None,
        },
        LogType::Timeout | LogType::TimedOutTimer => Event {
            kind: EventKind::Timer(TimerEvent::Timeout),
            time: None,
            number: None,
            name: message.to_owned(),
            values: None,
        },
        LogType::CoordinationMessage
        | LogType::StartDefault
        | LogType::DefaultEnded
        | LogType::ImplicitSend
        | LogType::Goto
        | LogType::Error
        | LogType::Create
        | LogType::Done
        | LogType::Activate
        | LogType::Message
        | LogType::LineMatched
        | LogType::LineNotMatched
        | LogType::OtherwiseEvent
        | LogType::ReceivedOnPco => Event {
            kind: EventKind::Ignored,
            time: None,
            number: None,
            name: format!("{:?} {}", logtype, message),
            values: None,
        },
    }
}

pub fn parse<E>(
    messages: impl Iterator<Item = Result<Message, E>>,
) -> impl Iterator<Item = Result<Event, E>> {
    messages.filter_map(|message| match message {
        Ok(Message::Log {
            message,
            logtype,
            time,
            ..
        }) => {
            let time = time.trim();

            let time = if !time.is_empty() {
                assert_eq!(&time[0..1], "+");
                assert_eq!(&time[time.len() - 3..], " ms");

                (&time[1..time.len() - 3]).parse::<u32>().ok()
            } else {
                None
            };

            Some(Ok(Event {
                time,
                ..parse_log_message(logtype, message)
            }))
        }
        Ok(_) => None,
        Err(e) => Some(Err(e)),
    })
}
