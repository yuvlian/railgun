use super::PacketError;
use byteorder::{BE, ByteOrder};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

pub const HEAD_MAGIC: [u8; 4] = [0x9D, 0x74, 0xC7, 0x14];
pub const TAIL_MAGIC: [u8; 4] = [0xD7, 0xA1, 0x52, 0xC8];

const HEAD_MAGIC_LEN: usize = 4;
const CMD_LEN: usize = 2;
const HEAD_SIZE_LEN: usize = 2;
const BODY_SIZE_LEN: usize = 4;
const TAIL_MAGIC_LEN: usize = 4;

const OVERHEAD: usize = HEAD_MAGIC_LEN + CMD_LEN + HEAD_SIZE_LEN + BODY_SIZE_LEN + TAIL_MAGIC_LEN;

const HM_START: usize = 0;
const HM_END: usize = HM_START + HEAD_MAGIC_LEN;

const CMD_START: usize = HM_END;
const CMD_END: usize = CMD_START + CMD_LEN;

const HS_START: usize = CMD_END;
const HS_END: usize = HS_START + HEAD_SIZE_LEN;

const BS_START: usize = HS_END;
const BS_END: usize = BS_START + BODY_SIZE_LEN;

#[derive(Debug, Clone)]
pub struct NetPacket {
    pub cmd: u16,
    pub head: Vec<u8>,
    pub body: Vec<u8>,
}

impl NetPacket {
    pub async fn write(&self, stream: &mut (impl AsyncWriteExt + Unpin)) -> std::io::Result<()> {
        stream.write_all(&HEAD_MAGIC).await?;
        stream.write_u16(self.cmd).await?;
        stream.write_u16(self.head.len() as u16).await?;
        stream.write_u32(self.body.len() as u32).await?;
        stream.write_all(&self.head).await?;
        stream.write_all(&self.body).await?;
        stream.write_all(&TAIL_MAGIC).await?;
        Ok(())
    }

    pub async fn read(stream: &mut (impl AsyncReadExt + Unpin)) -> std::io::Result<Self> {
        let mut head_magic = [0u8; 4];
        stream.read_exact(&mut head_magic).await?;
        if head_magic != HEAD_MAGIC {
            return Err(PacketError::InvalidHeadMagic.into());
        }

        let cmd = stream.read_u16().await?;

        let head_length = stream.read_u16().await? as usize;
        let body_length = stream.read_u32().await? as usize;

        let mut head = vec![0; head_length];
        stream.read_exact(&mut head).await?;

        let mut body = vec![0; body_length];
        stream.read_exact(&mut body).await?;

        let mut tail_magic = [0u8; 4];
        stream.read_exact(&mut tail_magic).await?;
        if tail_magic != TAIL_MAGIC {
            return Err(PacketError::InvalidTailMagic.into());
        }

        Ok(Self { cmd, head, body })
    }
}

impl TryFrom<&[u8]> for NetPacket {
    type Error = PacketError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < OVERHEAD {
            return Err(PacketError::TooShort);
        }

        if &data[HM_START..HM_END] != HEAD_MAGIC {
            return Err(PacketError::InvalidHeadMagic);
        }

        let cmd = BE::read_u16(&data[CMD_START..CMD_END]);
        let head_len = BE::read_u16(&data[HS_START..HS_END]) as usize;
        let body_len = BE::read_u32(&data[BS_START..BS_END]) as usize;

        let expected_len = OVERHEAD + head_len + body_len;

        if data.len() < expected_len {
            return Err(PacketError::SizeMismatch);
        }

        let head_start = BS_END;
        let head_end = head_start + head_len;

        let body_start = head_end;
        let body_end = body_start + body_len;

        let tail_start = body_end;
        let tail_end = tail_start + TAIL_MAGIC_LEN;

        if tail_end > data.len() {
            return Err(PacketError::SizeMismatch);
        }

        if &data[tail_start..tail_end] != TAIL_MAGIC {
            return Err(PacketError::InvalidTailMagic);
        }

        Ok(NetPacket {
            cmd,
            head: data[head_start..head_end].to_vec(),
            body: data[body_start..body_end].to_vec(),
        })
    }
}

impl From<NetPacket> for Vec<u8> {
    fn from(value: NetPacket) -> Self {
        let head_len = value.head.len();
        let body_len = value.body.len();

        let total_len = OVERHEAD + head_len + body_len;
        let mut out = vec![0u8; total_len];

        (&mut out[HM_START..HM_END]).copy_from_slice(&HEAD_MAGIC);
        BE::write_u16(&mut out[CMD_START..CMD_END], value.cmd);
        BE::write_u16(&mut out[HS_START..HS_END], head_len as u16);
        BE::write_u32(&mut out[BS_START..BS_END], body_len as u32);

        let head_start = BS_END;
        let head_end = head_start + head_len;

        let body_start = head_end;
        let body_end = body_start + body_len;

        let tail_start = body_end;
        let tail_end = tail_start + TAIL_MAGIC_LEN;

        (&mut out[head_start..head_end]).copy_from_slice(&value.head);
        (&mut out[body_start..body_end]).copy_from_slice(&value.body);
        (&mut out[tail_start..tail_end]).copy_from_slice(&TAIL_MAGIC);

        out
    }
}

impl From<NetPacket> for Box<[u8]> {
    fn from(value: NetPacket) -> Self {
        Vec::<u8>::from(value).into_boxed_slice()
    }
}
