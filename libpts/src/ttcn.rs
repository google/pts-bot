use nom::{
    branch::alt,
    bytes::complete::{take_until, take_while, take_while1},
    character::complete::{char, one_of},
    combinator::map,
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair, terminated},
    IResult,
};

use termion::{color, style};

use std::borrow::Cow;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum TTCNValue {
    Identifier(String),
    Integer(String),
    BitString(String),
    HexString(String),
    OctetString(String),
    CharString(String),
    Record(Vec<(String, TTCNValue)>),
    Array(Vec<TTCNValue>),
    Empty,
    AnyValue,
}

fn space(input: &str) -> IResult<&str, &str> {
    let chars = " \t\r\n";

    take_while(move |c| chars.contains(c))(input)
}

fn identifier(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '<' || c == '>')(input)
}

fn integer(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_numeric())(input)
}

fn charstring(input: &str) -> IResult<&str, &str> {
    delimited(char('"'), take_until("\""), char('"'))(input)
}

fn special_string(input: &str) -> IResult<&str, TTCNValue> {
    let (input, string) = delimited(char('\''), take_until("'"), char('\''))(input)?;

    let (input, tag) = one_of("HBO")(input)?;

    let value = match tag {
        'H' => TTCNValue::HexString(String::from(string)),
        'B' => TTCNValue::BitString(String::from(string)),
        'O' => TTCNValue::OctetString(String::from(string)),
        _ => unreachable!(),
    };

    Ok((input, value))
}

fn key_value(input: &str) -> IResult<&str, (&str, TTCNValue)> {
    separated_pair(
        preceded(space, identifier),
        preceded(space, char(':')),
        value,
    )(input)
}

fn record(input: &str) -> IResult<&str, Vec<(String, TTCNValue)>> {
    delimited(
        one_of("{["),
        map(
            separated_list0(preceded(space, char(',')), key_value),
            |vec| vec.into_iter().map(|(k, v)| (String::from(k), v)).collect(),
        ),
        preceded(space, one_of("}]")),
    )(input)
}

pub fn comma_separated_values(input: &str) -> IResult<&str, Vec<TTCNValue>> {
    let (input, _) = space(input)?;
    if input.is_empty() {
        Ok((input, vec![]))
    } else {
        terminated(separated_list0(preceded(space, char(',')), value), space)(input)
    }
}

fn array(input: &str) -> IResult<&str, Vec<TTCNValue>> {
    delimited(one_of("{["), comma_separated_values, one_of("}]"))(input)
}

fn value(input: &str) -> IResult<&str, TTCNValue> {
    preceded(
        space,
        alt((
            map(record, TTCNValue::Record),
            map(array, TTCNValue::Array),
            map(integer, |s| TTCNValue::Integer(String::from(s))),
            map(charstring, |s| TTCNValue::CharString(String::from(s))),
            special_string,
            map(char('?'), |_| TTCNValue::AnyValue),
            map(identifier, |s| TTCNValue::Identifier(String::from(s))),
            |input| Ok((input, TTCNValue::Empty)),
        )),
    )(input)
}

pub fn parse(input: &str) -> IResult<&str, TTCNValue> {
    value(input)
}

fn flatten<'k, 'v>(key: &'k str, value: &'v TTCNValue) -> (Cow<'k, str>, &'v TTCNValue) {
    match value {
        TTCNValue::Record(r) if r.len() == 1 => {
            let (rkey, rvalue) = r.into_iter().next().unwrap();
            let (rkey, rvalue) = flatten(rkey, rvalue);
            (Cow::Owned(format!("{}.{}", key, rkey)), rvalue)
        }
        _ => (Cow::Borrowed(key), value),
    }
}

impl fmt::Display for TTCNValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let padding = f.width().unwrap_or(0);
        match self {
            TTCNValue::Empty => write!(f, "<Empty>"),
            TTCNValue::Identifier(value) => {
                write!(f, "{}{}{}", color::Fg(color::Cyan), value, style::Reset)
            }
            TTCNValue::Integer(value) => {
                write!(f, "{}{}{}", color::Fg(color::Yellow), value, style::Reset)
            }
            TTCNValue::BitString(value) => write!(
                f,
                "{}0b{}{}",
                color::Fg(color::Magenta),
                value,
                style::Reset
            ),
            TTCNValue::HexString(value) => write!(
                f,
                "{}0x{}{}",
                color::Fg(color::LightMagenta),
                value,
                style::Reset
            ),
            TTCNValue::OctetString(value) => {
                write!(f, "{}0x{}{}", color::Fg(color::Red), value, style::Reset)
            }
            TTCNValue::CharString(value) => write!(
                f,
                "{}{:?}{}",
                color::Fg(color::LightGreen),
                value,
                style::Reset
            ),
            TTCNValue::AnyValue => write!(f, "{}?{}", style::Bold, style::Reset),
            TTCNValue::Record(record) => {
                if record.len() == 0 {
                    write!(f, "{{}}")
                } else {
                    write!(f, "{{\n")?;
                    for (key, value) in record {
                        let padding = padding + 2;
                        let (key, value) = flatten(key, value);
                        write!(f, "{:<1$}{key}: ", "", padding, key = key)?;
                        write!(f, "{:padding$}", value, padding = padding)?;
                        write!(f, ",\n")?;
                    }
                    write!(f, "{:<1$}}}", "", padding)
                }
            }
            TTCNValue::Array(array) => {
                write!(f, "[\n")?;
                for value in array {
                    let padding = padding + 2;
                    write!(f, "{:<1$}", "", padding)?;
                    write!(f, "{:padding$}", value, padding = padding)?;
                    write!(f, ",\n")?;
                }
                write!(f, "{:<1$}]", "", padding)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::comma_separated_values;
    use super::parse;
    use super::TTCNValue;

    #[test]
    fn test_identifier() {
        assert_eq!(
            parse("word"),
            Ok(("", TTCNValue::Identifier("word".to_owned())))
        );
        assert_eq!(
            parse("_word"),
            Ok(("", TTCNValue::Identifier("_word".to_owned())))
        );
        assert_eq!(
            parse("<word>"),
            Ok(("", TTCNValue::Identifier("<word>".to_owned())))
        );
        assert_eq!(parse("(word"), Ok(("(word", TTCNValue::Empty)));
    }

    #[test]
    fn test_integer() {
        assert_eq!(parse("42"), Ok(("", TTCNValue::Integer("42".to_owned()))));
    }

    #[test]
    #[ignore] // TODO: fix this test
    fn test_negative_integer() {
        assert_eq!(parse("-42"), Ok(("", TTCNValue::Integer("-42".to_owned()))));
    }

    #[test]
    fn test_bitstring() {
        assert_eq!(
            parse("'0101'B"),
            Ok(("", TTCNValue::BitString("0101".to_owned())))
        );
    }

    #[test]
    fn test_hexstring() {
        assert_eq!(
            parse("'2A'H"),
            Ok(("", TTCNValue::HexString("2A".to_owned())))
        );
    }

    #[test]
    fn test_octetstring() {
        assert_eq!(
            parse("'2A'O"),
            Ok(("", TTCNValue::OctetString("2A".to_owned())))
        );
    }

    #[test]
    fn test_charstring() {
        assert_eq!(
            parse(r#""word""#),
            Ok(("", TTCNValue::CharString("word".to_owned())))
        );
        assert_eq!(
            parse("\"wo\nrd\""),
            Ok(("", TTCNValue::CharString("wo\nrd".to_owned())))
        );
    }

    #[test]
    fn test_anyvalue() {
        assert_eq!(parse("?"), Ok(("", TTCNValue::AnyValue)));
    }

    #[test]
    fn test_parse() {
        let value = r#"{
           message:"{
             1010,
             %s,
             A2DP
           }Ifnecessary,
           takeactiontoaccepttheAVDTPStartoperationinitiatedbythetester.",
           signal:CM_SIGNAL_REQUEST,
           status:CM_STATUS_OK,
           style:MMI_Style_Ok_Cancel2
         }"#;

        let result = parse(value);

        println!("{:?}", result);

        let value = r#"{
           connection_handle:'00000040'O,
           avdtp:[
             cfm:[
               start:{
                 header:{
                   transaction_label:1,
                   packet_type:AVDTP_SINGLE_PACKET,
                   message_type:AVDTP_RESPONSE_ACCEPT,
                   nosp:OMIT,
                   rfa:0,
                   signal_identifier:AVDTP_START
                 }
               }
             ]
           ],
           param:OMIT
         }"#;

        let result = parse(value);

        println!("{:?}", result);
    }

    #[test]
    fn test_parse_comma_space() {
        let result = comma_separated_values("   ");
        assert_eq!(result, Ok(("", vec![])));
    }
}
