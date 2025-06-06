pub mod control_panel;
pub mod hsr;

#[derive(Debug)]
pub enum PacketError {
    TooShort,
    TooLong,
    InvalidHeadMagic,
    InvalidTailMagic,
    SizeMismatch,
    InvalidEncoding,
}

impl std::fmt::Display for PacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for PacketError {}

impl From<PacketError> for std::io::Error {
    fn from(err: PacketError) -> Self {
        use PacketError::*;
        let kind = match err {
            TooShort => std::io::ErrorKind::UnexpectedEof,
            TooLong => std::io::ErrorKind::InvalidData,
            SizeMismatch => std::io::ErrorKind::InvalidData,
            InvalidHeadMagic | InvalidTailMagic => std::io::ErrorKind::InvalidData,
            InvalidEncoding => std::io::ErrorKind::InvalidData,
        };
        std::io::Error::new(kind, err)
    }
}
