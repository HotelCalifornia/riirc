use bytes::Bytes;
use futures::{channel::mpsc::{self, UnboundedSender}, future::{self, Either, Future, FutureExt}, Sink, SinkExt, Stream, StreamExt};
use std::{error::Error, io, net::SocketAddr, string::String};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_util::codec::{BytesCodec, Decoder, FramedRead, FramedWrite};

pub struct Client {
    stream: std::pin::Pin<Box<dyn Stream<Item = Result<proto::Message, io::Error>>>>,
    sender: UnboundedSender<proto::Message>,
    user: proto::User,
}

pub type ClientRx = std::pin::Pin<Box<dyn Future<Output = Result<(), io::Error>> + Send>>;

impl Client {
    pub async fn new(addr: &SocketAddr, user: proto::User) -> Result<(Self, ClientRx), io::Error> {
        let stream = codec::ServerMessageCodec::default().framed(TcpStream::connect(addr));
        let (sink, stream) = stream.split();
        let (sender, receiver) = mpsc::unbounded();
        let sender_clone = sender.clone();
        let stream = stream.filter_map(move |message| {
            if let Ok(proto::Message {command: proto::Command::Cmd("PING"), params: params, ..}) = message {
                // message was a ping request, so respond to it and yield nothing
                let mut sender_clone = sender_clone.clone();
                Either::Left(async move {
                    match sender_clone.send(proto::Message {
                        tags: std::collections::HashMap::new(),
                        prefix: None,
                        command: proto::Command::from(String::from("PONG")),
                        params
                    }).await {
                        Ok(_) => None,
                        Err(err) => Some(err),
                    }
                })
            } else {
                // message was not a ping, so just yield it
                Either::Right(future::ready(Some(message)))
            }
        });
        // return client instance and a future that will yield messages from the server
        Ok((Client { stream: stream.boxed(), sender, user }, receiver.map(Ok).forward(sink).boxed()))
    }

    pub async fn send(&mut self, message: proto::Message) -> Result<(), Box<dyn Error>> {
        self.sender.send(message).await?;
        self.sender.flush().await?;
        Ok(())
    }

    pub async fn send_registration(&mut self) -> Result<(), Box<dyn Error>> {
        self.send(proto::Message {
            tags: std::collections::HashMap::new(),
            prefix: None,
            command: proto::Command::from(String::from("CAP")),
            params: vec![String::from("LS"), String::from("302")],
        }).await?;
        self.send(proto::Message {
            tags: std::collections::HashMap::new(),
            prefix: None,
            command: proto::Command::from(String::from("NICK")),
            params: vec![self.user.nick],
        }).await?;
        self.send(proto::Message {
            tags: std::collections::HashMap::new(),
            prefix: None,
            command: proto::Command::from(String::from("USER")),
            params: vec![self.user.name.ok_or(self.user.nick)?, String::from("0"), String::from("*"), self.user.real_name.ok_or("Anonymous")?],
        }).await?;
        self.send(proto::Message {
            tags: std::collections::HashMap::new(),
            prefix: None,
            command: proto::Command::from(String::from("CAP")),
            params: vec![String::from("END")],
        }).await?;
        Ok(())
    }
}

pub async fn connect(
    addr: &String,
    usr: proto::User,
    mut stdin: impl Stream<Item = Result<Bytes, io::Error>> + Unpin,
    mut stdout: impl Sink<self::proto::Message, Error = io::Error> + Unpin,
) -> Result<(), Box<dyn Error>> {
    println!(">> Connecting to {}:6697...", addr);
    let mut stream = TcpStream::connect(format!("{}:6667", addr)).await?;

    // connection registration begins
    // start with capability listing
    println!(">> CAP LS 302");
    stream.write(b"CAP LS 302\r\n").await?;

    // PASS command here if necessary

    println!(">> NICK {}", usr.nick);
    stream.write(format!("NICK {}\r\n", usr.nick).as_bytes()).await?;

    let username = usr.name.ok_or(usr.nick).unwrap();
    let real_name = usr.real_name.ok_or("Anonymous").unwrap();
    println!(">> USER {} 0 * :{}", username, real_name);
    stream.write(format!("USER {} 0 * :{}\r\n", username, real_name).as_bytes()).await?;

    // capability requests here if necessary

    // SASL setup here if negotiated

    // end capability negotiation
    println!(">> CAP END");
    stream.write(b"CAP END\r\n").await?;

    // pipe I/O to stdin/stdout
    let (r, w) = stream.split();

    let mut sink = FramedWrite::new(w, BytesCodec::new());

    let mut stream = FramedRead::new(r, self::codec::ServerMessageCodec::new())
        .filter_map(|i| match i {
            Ok(i) => {
                // println!("message: {:?}", i);
                // let command = i.command.clone();
                // // let sink = sink.clone();
                // match command {
                //     proto::Command::Cmd(c) => {
                //         match c.as_str() {
                //             "PING" => {
                //                 sink.send(Bytes::from(format!("PONG :{}", i.params.first().unwrap().as_str())));
                //                 sink.flush();
                //                 future::ready(None)
                //             },
                //             _ => {
                //                 future::ready(Some(i))
                //             }
                //         }
                //     },
                //     _ => {
                //         future::ready(Some(i))
                //     },
                // }
                future::ready(Some(i))
            },
            Err(e) => {
                eprintln!(">> ERROR: failed to read from socket: {}", e);
                future::ready(None)
            }
        })
        .map(Ok);
    
    match future::join(sink.send_all(&mut stdin), stdout.send_all(&mut stream)).await {
        (Err(e), _) | (_, Err(e)) => Err(e.into()),
        _ => Ok(()),
    }
}

pub mod codec;
pub mod proto;
pub mod transport;