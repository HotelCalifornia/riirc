use futures::{ready, Stream, task::Poll};
use tokio::{io::{AsyncRead, AsyncWrite}, net::TcpStream};
use tokio_util::codec::Framed;
use std::time;

pub struct Transport {
    stream: TcpStream,
}

impl AsyncRead for Transport {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [std::primitive::u8],
    ) -> Poll<std::io::Result<std::primitive::usize>> {
        TcpStream::poll_next(std::pin::Pin::new(self.get_mut().stream), cx, buf)
    }
}

impl AsyncWrite for Transport {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[std::primitive::u8],
    ) -> Poll<Result<std::primitive::usize, std::io::Error>> {
        TcpStream::poll_write(std::pin::Pin::new(self.get_mut().stream), cx, buf)
    }
    fn poll_flush(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Result<(), std::io::Error>> {
        TcpStream::poll_flush(std::pin::Pin::new(self.get_mut().stream), cx)
    }
    fn poll_shutdown(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Result<(), std::io::Error>> {
        TcpStream::poll_shutdown(std::pin::Pin::new(self.get_mut().stream), cx)
    }
}

// pub struct Transport<T> where T: AsyncRead + AsyncWrite + std::marker::Unpin {
//     inner: Framed<T, super::codec::ServerMessageCodec>,
//     ping: time::Instant,
// }



/*
impl<T> Transport<T> where T: AsyncRead + AsyncWrite + std::marker::Unpin {
    fn new(inner: Framed<T, super::codec::ServerMessageCodec>) -> Self {
        Transport {
            inner,
            ping: time::Instant::now(),
        }
    }
}

impl<T> Stream for Transport<T> where T: AsyncRead + AsyncWrite + std::marker::Unpin {
    type Item = super::proto::Message;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        // let zelf = self.get_mut();
        loop {
            match ready!(self.inner.poll_next(cx)) {
                Some(message) => match message.command {
                    super::proto::Command::Cmd(cmd) if cmd == "PING" => {
                        self.ping = time::Instant::now();
                        self.inner.start_send(super::proto::Message {
                            tags: std::collections::HashMap::new(),
                            prefix: None,
                            command: super::proto::Command::from(String::from("PONG")),
                            params: vec![message.params.first()],
                        });

                    }
                }
            }
        }
        // loop {
        //     match ready!(self.inner.poll_next(cx)) {
        //         Some(ref message) => {
        //             match message.command {
        //                 super::proto::Command(cmd) if cmd == "PING" => {

        //                 },
        //                 _ => 
        //             }
        //         },
        //         message=> return Async::ready(Some(message)),
        //     }
        // }
    }
}
*/