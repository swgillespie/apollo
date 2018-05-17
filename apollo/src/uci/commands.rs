// Copyright 2017-2018 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::str::FromStr;

/// UCI Commands that can be sent from the client to us.
#[derive(PartialEq, Eq, Debug)]
pub enum UciCommand {
    /// Tells the engine that the client intends to speak the UCI protocol.
    Uci,
    
    /// Tells the engine to enter or exit debug mode.
    Debug(bool)
}

#[derive(PartialEq, Eq, Debug)]
pub enum UciCommandParseError {
    EmptyString,
    ExtraTokens,
    MissingTokens,
    InvalidToken
}

macro_rules! bail_if {
    ($e:expr, $err:expr) => {
        if $e {
            return Err($err);
        }
    }
}

impl FromStr for UciCommand {
    type Err = UciCommandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_uci(tokens: &[&str]) -> Result<UciCommand, UciCommandParseError> {
            debug_assert_eq!(tokens[0], "uci");
            bail_if!(tokens.len() != 1, UciCommandParseError::ExtraTokens);
            Ok(UciCommand::Uci)
        }

        fn parse_debug(tokens: &[&str]) -> Result<UciCommand, UciCommandParseError> {
            debug_assert_eq!(tokens[0], "debug");
            bail_if!(tokens.len() > 2, UciCommandParseError::ExtraTokens);
            bail_if!(tokens.len() < 2, UciCommandParseError::MissingTokens);
            match tokens[1] {
                "on" => Ok(UciCommand::Debug(true)),
                "off" => Ok(UciCommand::Debug(false)),
                _ => Err(UciCommandParseError::InvalidToken)
            }
        }

        let tokens: Vec<_> = s.split_whitespace().collect();
        bail_if!(tokens.len() == 0, UciCommandParseError::EmptyString);
        match tokens[0] {
            "uci" => parse_uci(&tokens),
            "debug" => parse_debug(&tokens),
            _ => unimplemented!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! parse_test {
        ($name:ident, $str:expr, $cmd:expr) => {
            #[test]
            fn $name() {
                let cmd = $str.parse::<UciCommand>().unwrap();
                assert_eq!(cmd, $cmd);
            }
        }
    }

    macro_rules! parse_neg_test {
        ($name:ident, $str:expr, $err:expr) => {
            #[test]
            fn $name() {
                let err = concat!($str, "\n").parse::<UciCommand>().unwrap_err();
                assert_eq!(err, $err);
            }
        }
    }

    parse_neg_test!(empty_string,        "", UciCommandParseError::EmptyString);
    parse_test!    (uci,                 "uci", UciCommand::Uci);
    parse_neg_test!(uci_trailing_tokens, "uci foo bar", UciCommandParseError::ExtraTokens);
    parse_test!    (debug_on,            "debug on", UciCommand::Debug(true));
    parse_test!    (debug_off,           "debug off", UciCommand::Debug(false));
    parse_neg_test!(debug,               "debug", UciCommandParseError::MissingTokens);
    parse_neg_test!(debug_bad,           "debug foo", UciCommandParseError::InvalidToken);
    parse_neg_test!(debug_too_many,      "debug on on on", UciCommandParseError::ExtraTokens);
}
