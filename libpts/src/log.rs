use crate::pts::{BdAddr, LogType, Message};
use std::iter::Iterator;

#[derive(Debug)]
pub enum Event {
    EnterTestStep(String, usize),
    ExitTestStep(String, usize),
    Message(Message, usize),
}

pub fn parse(mut messages: impl Iterator<Item = Message>) -> (BdAddr, impl Iterator<Item = Event>) {
    let addr = messages
        .find_map(|message| {
            if let Message::Addr { value } = message {
                Some(value)
            } else {
                None
            }
        })
        .expect("Get Addr");

    let iter = messages.scan(Vec::new(), |stack, value| match value {
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
                Some(Event::EnterTestStep(test_name.to_owned(), stack.len() - 1))
            } else if let Some((_, test_name)) = message.split_once("Exit  Test Step") {
                let test_name = test_name.trim().to_owned();
                let last = stack.pop();
                assert_eq!(last.as_ref(), Some(&test_name));
                Some(Event::ExitTestStep(test_name, stack.len()))
            } else {
                Some(Event::Message(value, stack.len()))
            }
        }
        _ => Some(Event::Message(value, stack.len())),
    });

    (addr, iter)
}
