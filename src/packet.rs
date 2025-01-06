#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RconPacketType {
    ResponseValue,
    ExecCommand,
    AuthResponse,
    Auth,
}

impl Into<i32> for &RconPacketType {
    fn into(self) -> i32 {
        match self {
            RconPacketType::ResponseValue => 0,
            RconPacketType::ExecCommand => 2,
            RconPacketType::AuthResponse => 2,
            RconPacketType::Auth => 3,
        }
    }
}

impl TryFrom<i32> for RconPacketType {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(RconPacketType::ResponseValue),
            2 => Ok(RconPacketType::ExecCommand),
            3 => Ok(RconPacketType::Auth),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RconPacket {
    pub id: i32,
    pub r#type: RconPacketType,
    pub body: String,
}

impl RconPacket {
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = vec![0; 0];

        data.extend_from_slice(&i32::to_le_bytes(self.body.len() as i32 + 10));
        data.extend_from_slice(&i32::to_le_bytes(self.id));
        data.extend_from_slice(&i32::to_le_bytes(Into::<i32>::into(&self.r#type)));
        data.extend_from_slice(self.body.as_bytes());
        data.push(0);
        data.push(0);

        data
    }

    pub fn deserialize(data: &mut Vec<u8>) -> Option<RconPacket> {
        let length = i32::from_le_bytes(data[0..4].try_into().ok()?) as usize;

        if length + 4 > data.len() {
            return None;
        }

        let id = i32::from_le_bytes(data[4..8].try_into().ok()?);
        let r#type = i32::from_le_bytes(data[8..12].try_into().ok()?);
        let body = String::from_utf8_lossy(&data[12..(12 + length - 10)]).to_string();

        data.drain(..length + 4);

        Some(RconPacket {
            id,
            r#type: r#type.try_into().ok()?,
            body,
        })
    }
}
