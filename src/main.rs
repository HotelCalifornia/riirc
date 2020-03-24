#![warn(rust_2018_idioms)]

use futures::StreamExt;
use std::error::Error;
use tokio::io;
use tokio_util::codec::{FramedRead, FramedWrite};

mod irc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    use std::env;

    let mut args = env::args().skip(1).collect::<std::collections::VecDeque<_>>();

    // required parameters
    let server = args.pop_front().ok_or("usage: riirc server nick [username] [real name]")?;
    let nick = args.pop_front().ok_or("usage: riirc server nick [username] [real name]")?;

    // these are optional, and VecDeque::pop returns an Option<Item>
    let name = args.pop_front();
    let real_name = args.pop_front();
    

    let stdin = FramedRead::new(io::stdin(), irc::codec::CrLfDelimitedCodec::new()).map(|i| i.map(|bytes| bytes.freeze()));
    let stdout = FramedWrite::new(io::stdout(), irc::codec::ServerMessageCodec::new());

    irc::connect(&server, irc::proto::User::new(nick, name, real_name), stdin, stdout).await?;

    Ok(())
}
