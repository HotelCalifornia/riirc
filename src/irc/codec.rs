use tokio_util::codec::{Encoder, Decoder};
use bytes::{BufMut, BytesMut};

/// A simple [`Decoder`] implementation that splits up data into lines delimited by `<CR><LF>`
/// 
/// [`Decoder`]: tokiu_util::codec::Decoder
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct CrLfDelimitedCodec(());

impl CrLfDelimitedCodec {
    pub fn new() -> Self {
        Self(())
    }
}

impl Decoder for CrLfDelimitedCodec {
    type Item = BytesMut;
    type Error = std::io::Error;
    
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(last) = src.last_mut() {
            if *last == b'\n' {
                *last = b'\r';
                src.put_u8(b'\n');
                Ok(Some(src.split()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct ServerMessageCodec(());

impl ServerMessageCodec {
    pub fn new() -> Self {
        Self(())
    }
}

impl Decoder for ServerMessageCodec {
    type Item = super::proto::Message;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            // println!(">> need more bytes");
            Ok(None)
        } else {
            let src_str = String::from_utf8(src.to_vec()).unwrap();
            // println!(">> decoding {:?}", src);
            if let Some(i) = src_str.find("\r\n") {
                let mut f = src.split_to(i + 2);
                f = f.split_to(f.len() - "\r\n".len()); // sure hope this never goes < 0
                // println!(">> found frame at {}: {:?}", i, f);
                Ok(Some(super::proto::Message::from(f)))
            } else {
                // println!(">> no frame found yet");
                Ok(None)
            }
        }
    }
}

impl Encoder<super::proto::Message> for ServerMessageCodec {
    type Error = std::io::Error;
    fn encode(&mut self, item: super::proto::Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.clone_from(&BytesMut::from(item));
        println!(">> encoded {:?}", dst);
        Ok(())
    }
}