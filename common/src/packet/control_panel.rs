use super::PacketError;
use byteorder::{BE, ByteOrder};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

pub const HEAD_MAGIC: [u8; 4] = [0x17, 0x08, 0x19, 0x45]; // 17 agustus 1945 -amizing
pub const TAIL_MAGIC: [u8; 4] = *b"YVLN"; // yuvlian :3

const HEAD_MAGIC_LEN: usize = 4;
const USERNAME_SIZE_LEN: usize = 1;
const PASSWORD_SIZE_LEN: usize = 1;
const MAIN_CMD_LEN: usize = 1;
const SUB_CMD_LEN: usize = 1;
const BODY_SIZE_LEN: usize = 4;
const TAIL_MAGIC_LEN: usize = 4;

const OVERHEAD: usize = HEAD_MAGIC_LEN
    + USERNAME_SIZE_LEN
    + PASSWORD_SIZE_LEN
    + MAIN_CMD_LEN
    + SUB_CMD_LEN
    + BODY_SIZE_LEN
    + TAIL_MAGIC_LEN;

const HM_START: usize = 0;
const HM_END: usize = HM_START + HEAD_MAGIC_LEN;

const US_START: usize = HM_END;
const US_END: usize = US_START + USERNAME_SIZE_LEN;

const PS_START: usize = US_END;
const PS_END: usize = PS_START + PASSWORD_SIZE_LEN;

const MC_START: usize = PS_END;
const MC_END: usize = MC_START + MAIN_CMD_LEN;

const SC_START: usize = MC_END;
const SC_END: usize = SC_START + SUB_CMD_LEN;

const BS_START: usize = SC_END;
const BS_END: usize = BS_START + BODY_SIZE_LEN;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ControlPanelPacket {
    pub username: String,
    pub password: String,
    pub main_cmd: u8,
    pub sub_cmd: u8,
    pub body: Vec<u8>,
}

impl ControlPanelPacket {
    pub async fn write(&self, stream: &mut (impl AsyncWriteExt + Unpin)) -> std::io::Result<()> {
        let username_bytes = self.username.as_bytes();
        if username_bytes.len() > u8::MAX as usize {
            return Err(PacketError::TooLong.into());
        }
        let password_bytes = self.password.as_bytes();
        if password_bytes.len() > u8::MAX as usize {
            return Err(PacketError::TooLong.into());
        }

        stream.write_all(&HEAD_MAGIC).await?;
        stream.write_u8(username_bytes.len() as u8).await?;
        stream.write_u8(password_bytes.len() as u8).await?;
        stream.write_u8(self.main_cmd).await?;
        stream.write_u8(self.sub_cmd).await?;
        stream.write_u32(self.body.len() as u32).await?;
        stream.write_all(username_bytes).await?;
        stream.write_all(password_bytes).await?;
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

        let username_len = stream.read_u8().await? as usize;
        let password_len = stream.read_u8().await? as usize;
        let main_cmd = stream.read_u8().await?;
        let sub_cmd = stream.read_u8().await?;
        let body_len = stream.read_u32().await? as usize;

        let mut username_buf = vec![0u8; username_len];
        stream.read_exact(&mut username_buf).await?;
        let username = String::from_utf8(username_buf).map_err(|_| PacketError::InvalidEncoding)?;

        let mut password_buf = vec![0u8; password_len];
        stream.read_exact(&mut password_buf).await?;
        let password = String::from_utf8(password_buf).map_err(|_| PacketError::InvalidEncoding)?;

        let mut body = vec![0u8; body_len];
        stream.read_exact(&mut body).await?;

        let mut tail_magic = [0u8; 4];
        stream.read_exact(&mut tail_magic).await?;
        if tail_magic != TAIL_MAGIC {
            return Err(PacketError::InvalidTailMagic.into());
        }

        Ok(Self {
            username,
            password,
            main_cmd,
            sub_cmd,
            body,
        })
    }
}

impl TryFrom<&[u8]> for ControlPanelPacket {
    type Error = PacketError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < OVERHEAD {
            return Err(PacketError::TooShort);
        }

        if &data[HM_START..HM_END] != HEAD_MAGIC {
            return Err(PacketError::InvalidHeadMagic);
        }

        let username_len = data[US_START] as usize;
        let password_len = data[PS_START] as usize;
        let main_cmd = data[MC_START];
        let sub_cmd = data[SC_START];
        let body_len = BE::read_u32(&data[BS_START..BS_END]) as usize;

        let payload_start = BS_END;
        let username_end = payload_start + username_len;
        let password_end = username_end + password_len;
        let body_end = password_end + body_len;
        let tail_start = body_end;
        let tail_end = tail_start + TAIL_MAGIC_LEN;

        if tail_end > data.len() {
            return Err(PacketError::SizeMismatch);
        }

        let username = str::from_utf8(&data[payload_start..username_end])
            .map_err(|_| PacketError::InvalidEncoding)?
            .to_string();

        let password = str::from_utf8(&data[username_end..password_end])
            .map_err(|_| PacketError::InvalidEncoding)?
            .to_string();

        if &data[tail_start..tail_end] != TAIL_MAGIC {
            return Err(PacketError::InvalidTailMagic);
        }

        let body = data[password_end..body_end].to_vec();

        Ok(ControlPanelPacket {
            username,
            password,
            main_cmd,
            sub_cmd,
            body,
        })
    }
}

impl From<ControlPanelPacket> for Vec<u8> {
    fn from(packet: ControlPanelPacket) -> Self {
        let username_bytes = packet.username.as_bytes();
        let password_bytes = packet.password.as_bytes();
        let body_len = packet.body.len();

        let total_len = OVERHEAD + username_bytes.len() + password_bytes.len() + body_len;
        let mut buf = vec![0u8; total_len];

        (&mut buf[HM_START..HM_END]).copy_from_slice(&HEAD_MAGIC);
        buf[US_START] = username_bytes.len() as u8;
        buf[PS_START] = password_bytes.len() as u8;
        buf[MC_START] = packet.main_cmd;
        buf[SC_START] = packet.sub_cmd;
        BE::write_u32(&mut buf[BS_START..BS_END], body_len as u32);

        let mut offset = BS_END;
        buf[offset..offset + username_bytes.len()].copy_from_slice(username_bytes);
        offset += username_bytes.len();

        buf[offset..offset + password_bytes.len()].copy_from_slice(password_bytes);
        offset += password_bytes.len();

        buf[offset..offset + body_len].copy_from_slice(&packet.body);
        offset += body_len;

        buf[offset..offset + TAIL_MAGIC_LEN].copy_from_slice(&TAIL_MAGIC);

        buf
    }
}

impl From<ControlPanelPacket> for Box<[u8]> {
    fn from(packet: ControlPanelPacket) -> Self {
        Vec::<u8>::from(packet).into_boxed_slice()
    }
}
