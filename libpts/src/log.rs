use crate::pts::{LogType, Message};
use std::iter::Iterator;

#[derive(Debug)]
pub enum Event {
    EnterTestStep(String, usize),
    ExitTestStep(String, usize),
    Message(Message, usize),
}

pub fn parse<E>(
    messages: impl Iterator<Item = Result<Message, E>>,
) -> impl Iterator<Item = Result<Event, E>> {
    messages.scan(Vec::new(), |stack, value| {
        Some(value.map(|value| match value {
            Message::Log {
                logtype: LogType::Attach,
                ref message,
                ..
            } => {
                if let Some((_, test_name)) = message.split_once("Enter Test Step") {
                    let test_name = test_name.trim();
                    let test_name = test_name
                        .split_once(" ")
                        .map(|(first, _)| first)
                        .unwrap_or(test_name);
                    stack.push(test_name.to_owned());
                    Event::EnterTestStep(test_name.to_owned(), stack.len() - 1)
                } else if let Some((_, test_name)) = message.split_once("Exit  Test Step") {
                    let test_name = test_name.trim().to_owned();
                    let last = stack.pop();
                    assert_eq!(last.as_ref(), Some(&test_name));
                    Event::ExitTestStep(test_name, stack.len())
                } else {
                    Event::Message(value, stack.len())
                }
            }
            _ => Event::Message(value, stack.len()),
        }))
    })
}
