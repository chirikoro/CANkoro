/// DBC file parser using nom
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::{char, line_ending, multispace0, multispace1, space0, space1},
    combinator::{map, opt, value},
    multi::{many0, separated_list0},
    number::complete::double,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

use super::message::DbcMessage;
use super::signal::{ByteOrder, DbcSignal};
use std::collections::HashMap;

/// Parse a quoted string
fn quoted_string(input: &str) -> IResult<&str, &str> {
    delimited(char('"'), take_until("\""), char('"'))(input)
}

/// Parse an identifier (C-style)
fn identifier(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)
}

/// Parse an integer (possibly negative)
fn integer(input: &str) -> IResult<&str, i64> {
    let (input, sign) = opt(char('-'))(input)?;
    let (input, digits) = take_while1(|c: char| c.is_ascii_digit())(input)?;
    let val: i64 = digits.parse().unwrap_or(0);
    Ok((input, if sign.is_some() { -val } else { val }))
}

/// Parse an unsigned integer
fn unsigned_integer(input: &str) -> IResult<&str, u64> {
    let (input, digits) = take_while1(|c: char| c.is_ascii_digit())(input)?;
    let val: u64 = digits.parse().unwrap_or(0);
    Ok((input, val))
}

/// Parse a float number
fn float_number(input: &str) -> IResult<&str, f64> {
    // Try parsing with nom's double, which handles scientific notation
    let (input, _) = space0(input)?;
    double(input)
}

/// Parse byte order: 0 = BigEndian (Motorola), 1 = LittleEndian (Intel)
fn byte_order(input: &str) -> IResult<&str, ByteOrder> {
    alt((
        value(ByteOrder::BigEndian, char('0')),
        value(ByteOrder::LittleEndian, char('1')),
    ))(input)
}

/// Parse value type: + = unsigned, - = signed
fn value_type(input: &str) -> IResult<&str, bool> {
    alt((value(true, char('-')), value(false, char('+'))))(input)
}

/// Parse a signal definition:
/// SG_ signal_name : start_bit|length@byte_order value_type (factor,offset) [min|max] "unit" receivers
fn parse_signal(input: &str) -> IResult<&str, DbcSignal> {
    let (input, _) = space0(input)?;
    let (input, _) = tag("SG_")(input)?;
    let (input, _) = space1(input)?;
    let (input, name) = identifier(input)?;

    // Optional multiplexer indicator
    let (input, _) = space0(input)?;
    let (input, _mux) = opt(alt((
        map(
            tuple((char('m'), take_while1(|c: char| c.is_ascii_digit()))),
            |_| (),
        ),
        value((), char('M')),
    )))(input)?;

    let (input, _) = space0(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = space0(input)?;

    // start_bit|length
    let (input, start_bit) = unsigned_integer(input)?;
    let (input, _) = char('|')(input)?;
    let (input, bit_length) = unsigned_integer(input)?;

    // @byte_order value_type
    let (input, _) = char('@')(input)?;
    let (input, bo) = byte_order(input)?;
    let (input, is_signed) = value_type(input)?;

    let (input, _) = space0(input)?;

    // (factor,offset)
    let (input, _) = char('(')(input)?;
    let (input, factor) = float_number(input)?;
    let (input, _) = char(',')(input)?;
    let (input, offset) = float_number(input)?;
    let (input, _) = char(')')(input)?;

    let (input, _) = space0(input)?;

    // [min|max]
    let (input, _) = char('[')(input)?;
    let (input, min) = float_number(input)?;
    let (input, _) = char('|')(input)?;
    let (input, max) = float_number(input)?;
    let (input, _) = char(']')(input)?;

    let (input, _) = space0(input)?;

    // "unit"
    let (input, unit) = quoted_string(input)?;

    let (input, _) = space0(input)?;

    // receivers (comma-separated)
    let (input, receivers_str) =
        take_while(|c: char| c != '\n' && c != '\r')(input)?;
    let receivers: Vec<String> = receivers_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok((
        input,
        DbcSignal {
            name: name.to_string(),
            start_bit: start_bit as u32,
            bit_length: bit_length as u32,
            byte_order: bo,
            is_signed,
            factor,
            offset,
            min,
            max,
            unit: unit.to_string(),
            receivers,
            value_descriptions: None,
        },
    ))
}

/// Parse a message definition:
/// BO_ message_id message_name: message_length transmitter
///   SG_ ...
fn parse_message(input: &str) -> IResult<&str, DbcMessage> {
    let (input, _) = tag("BO_")(input)?;
    let (input, _) = space1(input)?;
    let (input, id) = unsigned_integer(input)?;
    let (input, _) = space1(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = space0(input)?;
    let (input, dlc) = unsigned_integer(input)?;
    let (input, _) = space1(input)?;
    let (input, transmitter) = take_while(|c: char| c != '\n' && c != '\r')(input)?;

    // Parse signals (one per line, indented with spaces)
    let (input, _) = opt(line_ending)(input)?;
    let (input, signals) = many0(terminated(parse_signal, opt(line_ending)))(input)?;

    // Handle extended IDs (bit 31 set in DBC means extended)
    let can_id = if id & 0x80000000 != 0 {
        id as u32 & 0x1FFFFFFF
    } else {
        id as u32
    };

    Ok((
        input,
        DbcMessage {
            id: can_id,
            name: name.to_string(),
            dlc: dlc as u8,
            transmitter: transmitter.trim().to_string(),
            signals,
        },
    ))
}

/// Parse value descriptions:
/// VAL_ message_id signal_name value1 "desc1" value2 "desc2" ... ;
fn parse_value_descriptions(input: &str) -> IResult<&str, (u32, String, HashMap<i64, String>)> {
    let (input, _) = tag("VAL_")(input)?;
    let (input, _) = space1(input)?;
    let (input, msg_id) = unsigned_integer(input)?;
    let (input, _) = space1(input)?;
    let (input, signal_name) = identifier(input)?;

    let mut map = HashMap::new();
    let mut remaining = input;

    loop {
        let (input, _) = space0(remaining)?;
        if input.starts_with(';') {
            remaining = &input[1..];
            break;
        }
        match integer(input) {
            Ok((input, val)) => {
                let (input, _) = space1(input)?;
                let (input, desc) = quoted_string(input)?;
                map.insert(val, desc.to_string());
                remaining = input;
            }
            Err(_) => {
                // Skip to end of line
                let pos = input.find(';').unwrap_or(input.len());
                remaining = if pos < input.len() {
                    &input[pos + 1..]
                } else {
                    &input[pos..]
                };
                break;
            }
        }
    }

    Ok((remaining, (msg_id as u32, signal_name.to_string(), map)))
}

/// Parse an entire DBC file
pub fn parse_dbc(content: &str) -> Result<Vec<DbcMessage>, String> {
    let mut messages = Vec::new();
    let mut value_descs: Vec<(u32, String, HashMap<i64, String>)> = Vec::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.starts_with("BO_ ") {
            // Collect message block (until next empty line or next keyword)
            let mut block = String::from(lines[i]);
            i += 1;
            while i < lines.len() {
                let next_line = lines[i];
                let trimmed = next_line.trim();
                if trimmed.starts_with("SG_ ") || trimmed.starts_with(" SG_ ") {
                    block.push('\n');
                    block.push_str(next_line);
                    i += 1;
                } else {
                    break;
                }
            }
            match parse_message(&block) {
                Ok((_, msg)) => messages.push(msg),
                Err(e) => {
                    log::warn!("Failed to parse message: {:?} in block: {}", e, &block[..block.len().min(100)]);
                }
            }
        } else if line.starts_with("VAL_ ") {
            // Collect value description (may span multiple lines until ;)
            let mut block = String::from(line);
            if !line.contains(';') {
                i += 1;
                while i < lines.len() {
                    block.push(' ');
                    block.push_str(lines[i].trim());
                    if lines[i].contains(';') {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
            } else {
                i += 1;
            }
            match parse_value_descriptions(&block) {
                Ok((_, vd)) => value_descs.push(vd),
                Err(e) => {
                    log::warn!("Failed to parse value description: {:?}", e);
                }
            }
        } else {
            i += 1;
        }
    }

    // Apply value descriptions to signals
    for (msg_id, signal_name, descs) in value_descs {
        // Handle extended ID in VAL_
        let lookup_id = if msg_id & 0x80000000 != 0 {
            msg_id & 0x1FFFFFFF
        } else {
            msg_id
        };
        for msg in &mut messages {
            if msg.id == lookup_id {
                for sig in &mut msg.signals {
                    if sig.name == signal_name {
                        sig.value_descriptions = Some(descs.clone());
                    }
                }
            }
        }
    }

    Ok(messages)
}
