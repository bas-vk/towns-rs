use crate::TownsError;
use alloy_primitives::{Address, FixedBytes};
use std::fmt;
use std::fmt::Formatter;

pub const STREAM_ID_LEN: usize = 32;

pub const SPACE_STREAM_ID_PREFIX: u8 = 0x10;
pub const CHANNEL_STREAM_ID_PREFIX: u8 = 0x20;
pub const GDM_CHANNEL_STREAM_ID_PREFIX: u8 = 0x77;
pub const DM_CHANNEL_STREAM_ID_PREFIX: u8 = 0x88;
pub const USER_INBOX_STREAM_ID_PREFIX: u8 = 0xa1;
pub const USER_SETTINGS_STREAM_ID_PREFIX: u8 = 0xa5;
pub const USER_STREAM_ID_PREFIX: u8 = 0xa8;
pub const USER_METADATA_STREAM_ID_PREFIX: u8 = 0xad;
pub const MEDIA_STREAM_ID_PREFIX: u8 = 0xff;

/// StreamId uniquely identifies a stream.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum StreamId {
    /// StreamId for a stream that contains user meta data
    UserMetaDataKey(FixedBytes<STREAM_ID_LEN>),
    /// StreamId for a stream that contains events for a user
    UserInbox(FixedBytes<STREAM_ID_LEN>),
    /// StreamId for a stream that contains personal events for a user
    User(FixedBytes<STREAM_ID_LEN>),
    /// StreamId for a stream that contains user specific settings
    UserSettings(FixedBytes<STREAM_ID_LEN>),
    /// StreamId for a stream that contains media events
    Media(FixedBytes<STREAM_ID_LEN>),
    /// StreamId for a stream that contains events for a channel
    Channel(FixedBytes<STREAM_ID_LEN>),
    /// StreamId for a stream that contains events for direct message channel
    DmChannel(FixedBytes<STREAM_ID_LEN>),
    /// StreamId for a stream that contains events for a group message channel
    GdmChannel(FixedBytes<STREAM_ID_LEN>),
    /// StreamId for a stream that contains events for a space
    Space(FixedBytes<STREAM_ID_LEN>),
}

impl StreamId {
    pub fn stream_type(&self) -> u8 {
        match self {
            StreamId::UserMetaDataKey(_) => USER_METADATA_STREAM_ID_PREFIX,
            StreamId::UserInbox(_) => USER_INBOX_STREAM_ID_PREFIX,
            StreamId::User(_) => USER_STREAM_ID_PREFIX,
            StreamId::UserSettings(_) => USER_SETTINGS_STREAM_ID_PREFIX,
            StreamId::Media(_) => MEDIA_STREAM_ID_PREFIX,
            StreamId::Channel(_) => CHANNEL_STREAM_ID_PREFIX,
            StreamId::DmChannel(_) => DM_CHANNEL_STREAM_ID_PREFIX,
            StreamId::GdmChannel(_) => GDM_CHANNEL_STREAM_ID_PREFIX,
            StreamId::Space(_) => SPACE_STREAM_ID_PREFIX,
        }
    }
    pub fn as_fixed_bytes32(&self) -> FixedBytes<STREAM_ID_LEN> {
        match self {
            StreamId::UserMetaDataKey(raw) => raw.clone(),
            StreamId::UserInbox(raw) => raw.clone(),
            StreamId::User(raw) => raw.clone(),
            StreamId::UserSettings(raw) => raw.clone(),
            StreamId::Media(raw) => raw.clone(),
            StreamId::Channel(raw) => raw.clone(),
            StreamId::DmChannel(raw) => raw.clone(),
            StreamId::GdmChannel(raw) => raw.clone(),
            StreamId::Space(raw) => raw.clone(),
        }
    }

    pub fn try_from_short(from: &[u8]) -> Result<Self, TownsError> {
        if from.len() != 21 {
            return Err(TownsError::InvalidArgument("stream_id"));
        }

        let mut id = FixedBytes::<STREAM_ID_LEN>::new([0; STREAM_ID_LEN]);
        id[0..=20].copy_from_slice(from);

        match from[0] {
            USER_METADATA_STREAM_ID_PREFIX => Ok(StreamId::UserMetaDataKey(id)),
            USER_INBOX_STREAM_ID_PREFIX => Ok(StreamId::UserInbox(id)),
            USER_STREAM_ID_PREFIX => Ok(StreamId::User(id)),
            USER_SETTINGS_STREAM_ID_PREFIX => Ok(StreamId::UserSettings(id)),
            _ => Err(TownsError::InvalidArgument("stream id")),
        }
    }

    pub fn try_from_long(from: &[u8]) -> Result<Self, TownsError> {
        if from.len() != STREAM_ID_LEN {
            return Err(TownsError::InvalidArgumentWithValue(
                "stream_id",
                hex::encode(from),
            ));
        }

        let id = FixedBytes::<STREAM_ID_LEN>::from_slice(from);

        match from[0] {
            MEDIA_STREAM_ID_PREFIX => Ok(StreamId::Media(id)),
            CHANNEL_STREAM_ID_PREFIX => Ok(StreamId::Channel(id)),
            DM_CHANNEL_STREAM_ID_PREFIX => Ok(StreamId::DmChannel(id)),
            GDM_CHANNEL_STREAM_ID_PREFIX => Ok(StreamId::GdmChannel(id)),
            SPACE_STREAM_ID_PREFIX => Ok(StreamId::Space(id)),
            USER_INBOX_STREAM_ID_PREFIX => Ok(StreamId::Space(id)),
            USER_SETTINGS_STREAM_ID_PREFIX => Ok(StreamId::Space(id)),
            USER_STREAM_ID_PREFIX => Ok(StreamId::Space(id)),
            USER_METADATA_STREAM_ID_PREFIX => Ok(StreamId::Space(id)),
            _ => Err(TownsError::InvalidArgumentWithValue(
                "stream_id",
                hex::encode(from),
            )),
        }
    }

    pub fn user_stream_from_addr(addr: &Address) -> StreamId {
        let mut id = FixedBytes::<STREAM_ID_LEN>::new([0u8; STREAM_ID_LEN]);
        id[0] = USER_STREAM_ID_PREFIX;
        id[1..=20].copy_from_slice(addr.as_slice());
        StreamId::User(id)
    }

    pub fn user_settings_stream_from_addr(addr: &Address) -> StreamId {
        let mut id = FixedBytes::<STREAM_ID_LEN>::new([0u8; STREAM_ID_LEN]);
        id[0] = USER_SETTINGS_STREAM_ID_PREFIX;
        id[1..=20].copy_from_slice(addr.as_slice());
        StreamId::User(id)
    }

    pub fn user_inbox_stream_from_addr(addr: &Address) -> StreamId {
        let mut id = FixedBytes::<STREAM_ID_LEN>::new([0u8; STREAM_ID_LEN]);
        id[0] = USER_INBOX_STREAM_ID_PREFIX;
        id[1..=20].copy_from_slice(addr.as_slice());
        StreamId::User(id)
    }

    pub fn user_metadata_key_stream_from_addr(addr: &Address) -> StreamId {
        let mut id = FixedBytes::<STREAM_ID_LEN>::new([0u8; STREAM_ID_LEN]);
        id[0] = USER_METADATA_STREAM_ID_PREFIX;
        id[1..=20].copy_from_slice(addr.as_slice());
        StreamId::User(id)
    }
}

impl std::str::FromStr for StreamId {
    type Err = TownsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        StreamId::try_from(s)
    }
}

impl TryFrom<&[u8]> for StreamId {
    type Error = TownsError;

    fn try_from(from: &[u8]) -> Result<Self, Self::Error> {
        match from.len() {
            21 => StreamId::try_from_short(from),
            32 => StreamId::try_from_long(from),
            _ => Err(TownsError::InvalidArgument("stream_id")),
        }
    }
}

impl From<&alloy_primitives::FixedBytes<32>> for StreamId {
    fn from(from: &alloy_primitives::FixedBytes<32>) -> Self {
        StreamId::try_from(from.as_slice()).unwrap()
    }
}

impl TryFrom<&str> for StreamId {
    type Error = TownsError;

    fn try_from(mut from: &str) -> Result<Self, Self::Error> {
        if from.len() > 2 && from.starts_with("0x") || from.starts_with("0X") {
            from = &from[2..];
        }

        let raw = hex::decode(from)
            .map_err(|op| TownsError::InvalidArgumentWithValue("stream_id", op.to_string()))?;

        Ok(StreamId::try_from(raw.as_slice())
            .map_err(|_| TownsError::InvalidArgumentWithValue("stream_id", hex::encode(from)))?)
    }
}

impl Into<Vec<u8>> for StreamId {
    fn into(self) -> Vec<u8> {
        match self {
            StreamId::UserMetaDataKey(raw) => {
                let mut result = raw.to_vec();
                result.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
                result
            }
            StreamId::UserInbox(raw) => {
                let mut result = raw.to_vec();
                result.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
                result
            }
            StreamId::User(raw) => {
                let mut result = raw.to_vec();
                result.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
                result
            }
            StreamId::UserSettings(raw) => {
                let mut result = raw.to_vec();
                result.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
                result
            }
            StreamId::Media(raw) => raw.to_vec(),
            StreamId::Channel(raw) => raw.to_vec(),
            StreamId::DmChannel(raw) => raw.to_vec(),
            StreamId::GdmChannel(raw) => raw.to_vec(),
            StreamId::Space(raw) => raw.to_vec(),
        }
    }
}

impl fmt::Display for StreamId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StreamId::UserMetaDataKey(raw) => write!(f, "0x{}", hex::encode(raw)),
            StreamId::UserInbox(raw) => write!(f, "0x{}", hex::encode(raw)),
            StreamId::User(raw) => write!(f, "0x{}", hex::encode(raw)),
            StreamId::UserSettings(raw) => write!(f, "0x{}", hex::encode(raw)),
            StreamId::Media(raw) => write!(f, "0x{}", hex::encode(raw)),
            StreamId::Channel(raw) => write!(f, "0x{}", hex::encode(raw)),
            StreamId::DmChannel(raw) => write!(f, "0x{}", hex::encode(raw)),
            StreamId::GdmChannel(raw) => write!(f, "0x{}", hex::encode(raw)),
            StreamId::Space(raw) => write!(f, "0x{}", hex::encode(raw)),
        }
    }
}

impl AsRef<FixedBytes<STREAM_ID_LEN>> for StreamId {
    fn as_ref(&self) -> &FixedBytes<STREAM_ID_LEN> {
        match self {
            StreamId::UserMetaDataKey(raw) => raw,
            StreamId::UserInbox(raw) => raw,
            StreamId::User(raw) => raw,
            StreamId::UserSettings(raw) => raw,
            StreamId::Media(raw) => raw,
            StreamId::Channel(raw) => raw,
            StreamId::DmChannel(raw) => raw,
            StreamId::GdmChannel(raw) => raw,
            StreamId::Space(raw) => raw,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_stream_id() {
        let e =
            StreamId::try_from(Vec::from([SPACE_STREAM_ID_PREFIX, 0x2]).as_slice()).unwrap_err();
        if !matches!(TownsError::InvalidArgument("stream_id"), e) {
            panic!("expected InvalidArgument error");
        }
    }

    #[test]
    fn parse_too_short_stream_id() {
        let e =
            StreamId::try_from(Vec::from([SPACE_STREAM_ID_PREFIX, 0x2]).as_slice()).unwrap_err();
        if !matches!(TownsError::InvalidArgument("stream_id"), e) {
            panic!("expected InvalidArgument error");
        }
    }

    #[test]
    fn parse_short_stream_id() {
        let hex_encoded = "ad0100000000000000000000000000000000000009";
        let parsed = StreamId::try_from(hex_encoded).unwrap();
        let mut exp = FixedBytes::<STREAM_ID_LEN>::ZERO;
        exp[0] = USER_METADATA_STREAM_ID_PREFIX;
        exp[1] = 0x01;
        exp[20] = 0x09;
        assert_eq!(StreamId::UserMetaDataKey(exp), parsed)
    }

    #[test]
    fn parse_long_stream_id() {
        let hex_encoded = "7701000000000000000000000000000000000000000000000000000000000009";
        let parsed = StreamId::try_from(hex_encoded).unwrap();
        let mut exp = FixedBytes::<STREAM_ID_LEN>::ZERO;
        exp[0] = GDM_CHANNEL_STREAM_ID_PREFIX;
        exp[1] = 0x01;
        exp[STREAM_ID_LEN - 1] = 0x09;
        assert_eq!(StreamId::GdmChannel(exp), parsed)
    }
}
