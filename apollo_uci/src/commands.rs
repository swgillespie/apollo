// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::io::{Write, Error};
use traits::ToUciWire;

/// Enum representing all of the possible commands that are allowed in
/// the universal chess interface (UCI) protocol.
#[derive(Debug)]
pub enum Command {
    /// Sent by the client to tell the server that it intends to use
    /// the UCI protocol.
    Uci,

    /// Sent by the client to request the server to send additional
    /// debug info through `info string`.
    DebugOn,

    /// Sent by the client to request the server to stop sending additional
    /// debug info.
    DebugOff,

    /// Sent by the client whenever the client wants to synchronize with
    /// the server. When the client has sent commands that may take some
    /// time to complete, it can send `isready`, and the server will reply
    /// `readyok` once these tasks have completed.
    IsReady,

    /// Sent to the server whenever the client wants to change internal
    /// parameters of the server.
    SetOption {
        /// The name of the option. Not case sensitive.
        name: String,

        /// The value of the option.
        value: String,
    },

    /// Sent by the client if the server has requested registration through
    /// the `registration error` command and the client will register later.
    RegisterLater,

    /// Sent by the client if the server has requested registration through
    /// the `registration error` command and the client is providing
    /// registration information.
    Register { name: String, code: String },

    /// Sent by the client to indicate that the next search will be from a
    /// new game and not from an existing position.
    UciNewGame,

    /// Sent by the client to tell the server to set up the position described
    /// by `fen` (or the starting position, if `fen` is not given). The engine
    /// is then instructed to apply the given moves to the position.
    Position {
        fen: Option<String>,
        moves: Vec<String>,
    },

    /// Sent by the client to initiate a move search.
    Go {
        search_moves: Vec<String>,
        wtime: String,
        btime: String,
        winc: String,
        binc: String,
        movestogo: String,
        depth: String,
        nodes: String,
        mate: String,
        movetime: String,
        infinite: bool,
        ponder: bool,
    },

    /// Stop calculating as soon as possible.
    Stop,

    /// Sent by the client to indicate that the user played the ponder move.
    Ponderhit,

    /// Quit the program as soon as possible.
    Quit,

    /// Sent by the server to identify itself with the client.
    IdName { name: String },

    /// Sent by the server to identify its authors to the client.
    IdAuthor { authors: String },

    /// Sent by the server to respond to `isready` in the affirmative.
    ReadyOk,

    /// Sent by the server once it has introduced itself to the client
    /// and is ready to receive UCI commands.
    UciOk
}

impl ToUciWire for Command {
    fn to_uci_wire<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        match *self {
            Command::Uci => writeln!(writer, "uci"),
            Command::DebugOn => writeln!(writer, "debug on"),
            Command::DebugOff => writeln!(writer, "debug off"),
            Command::IsReady => writeln!(writer, "isready"),
            Command::SetOption { ref name, ref value } => {
                writeln!(writer, "setoption name {} value {}", name, value)
            }
            Command::RegisterLater => writeln!(writer, "register later"),
            Command::Register { ref name, ref code } => {
                writeln!(writer, "register name {} code {}", name, code)
            }
            Command::UciNewGame => writeln!(writer, "ucinewgame"),
            Command::Position { ref fen, ref moves } => {
                writeln!(writer, "position ")?;
                if let Some(ref fenstr) = *fen {
                    writeln!(writer, "{} ", fenstr)?;
                } else {
                    writeln!(writer, "startpos")?;
                }

                if !moves.is_empty() {
                    writeln!(writer, " {}", moves.join(" "))?;
                }

                Ok(())
            }
            Command::Stop => writeln!(writer, "stop"),
            Command::Ponderhit => writeln!(writer, "ponderhit"),
            Command::Quit => writeln!(writer, "quit"),
            Command::IdAuthor { ref authors } => writeln!(writer, "id author {}", authors),
            Command::IdName { ref name } => writeln!(writer, "id name {}", name),
            Command::ReadyOk => writeln!(writer, "readyok"),
            Command::UciOk => writeln!(writer, "uciok"),
            _ => unimplemented!(),
        }
    }
}