// Copyright 2025 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::char,
    combinator::{opt, recognize},
    error::{make_error, ErrorKind},
    multi::separated_list1,
    sequence::{delimited, preceded},
    Err, IResult,
};

fn space(input: &str) -> IResult<&str, &str> {
    let chars = " \t\r\n";
    take_while(move |c| chars.contains(c))(input)
}

fn comma(input: &str) -> IResult<&str, &str> {
    recognize(delimited(opt(space), char(','), opt(space)))(input)
}

fn integer(input: &str) -> IResult<&str, &str> {
    recognize(preceded(
        opt(char('-')),
        take_while1(|c: char| c.is_numeric()),
    ))(input)
}

fn range(input: &str) -> IResult<&str, &str> {
    recognize(delimited(integer, char('-'), integer))(input)
}

fn string(input: &str) -> IResult<&str, &str> {
    recognize(delimited(char('"'), take_until("\""), char('"')))(input)
}

fn sequence(input: &str) -> IResult<&str, &str> {
    recognize(separated_list1(comma, opt(value)))(input)
}

fn value(input: &str) -> IResult<&str, &str> {
    alt((
        range,
        integer,
        string,
        recognize(delimited(char('('), sequence, char(')'))),
    ))(input)
}

// List only commands which accept parameters.
const COMMANDS: &[&str] = &[
    "+CRING:", "+CREG:", "+CLIP:", "+COLP:", "+CCWA:", "+CUSB:", "+CCCM:", "+CSSI:", "+CSSU:",
    "+CBC:", "+CSQ:", "+CIEV:", "+CIND:", "+CCWV:", "+CTZV:", "+CGREG:", "+CMTI:", "+CMT:",
    "+CDSI:", "+CBM:", "+BINP:", "+CNUM:", "+COPS:",
];

/// Parse a subset of AT command formats.
pub fn parse(input: &str) -> IResult<&str, &str> {
    for c in COMMANDS {
        let result = recognize(delimited(tag(*c), opt(space), sequence))(input);
        if result.is_ok() {
            return result;
        }
    }
    Err(Err::Error(make_error(input, ErrorKind::Alt)))
}

#[cfg(test)]
mod test {
    use super::parse;

    #[test]
    fn test_at_cind() {
        let input = r#"+CIND:("service",
                           (0,
                           1)),
                           ("call",
                           (0,
                           1)),
                           ("callsetup",
                           (0-3)),
                           ("callheld",
                           (0-2)),
                           ("signal",
                           (0-5)),
                           ("roam",
                           (0-1)),
                           ("battchg",
                           (0-5))"#;
        assert_eq!(parse(input), Ok(("", input)));
    }

    #[test]
    fn test_at_cnum() {
        let input = r#"+CNUM:,
                           "1234567",
                           129,
                           ,
                           4"#;
        assert_eq!(parse(input), Ok(("", input)));
    }

    #[test]
    fn test_at_string() {
        let input = r#"+CBM:"service""#;
        assert_eq!(parse(input), Ok(("", input)));
    }

    #[test]
    fn test_at_integer() {
        let input = r#"+CBM:123"#;
        assert_eq!(parse(input), Ok(("", input)));
    }

    #[test]
    fn test_at_range() {
        let input = r#"+CBM:1-20"#;
        assert_eq!(parse(input), Ok(("", input)));
    }

    #[test]
    fn test_at_sequence() {
        let input = r#"+CBM:,1,2,,3"#;
        assert_eq!(parse(input), Ok(("", input)));
    }

    #[test]
    fn test_at_list() {
        let input = r#"+CBM:(1,2,3)"#;
        assert_eq!(parse(input), Ok(("", input)));
    }
}
