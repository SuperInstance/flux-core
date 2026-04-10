//! A2A (Agent-to-Agent) protocol message types and serialization.

use crate::error::{Error, Result};
use core::fmt;

/// A2A message types corresponding to the opcodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    /// Tell message (one-way communication)
    Tell = 1,

    /// Ask message (request-response)
    Ask = 2,

    /// Delegate message (task delegation)
    Delegate = 3,

    /// Broadcast message (one-to-many)
    Broadcast = 4,
}

impl MessageType {
    /// Create a MessageType from a u8.
    pub fn from_u8(value: u8) -> Result<MessageType> {
        match value {
            1 => Ok(MessageType::Tell),
            2 => Ok(MessageType::Ask),
            3 => Ok(MessageType::Delegate),
            4 => Ok(MessageType::Broadcast),
            _ => Err(Error::InvalidMessageType(value)),
        }
    }

    /// Convert to u8.
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageType::Tell => write!(f, "TELL"),
            MessageType::Ask => write!(f, "ASK"),
            MessageType::Delegate => write!(f, "DELEGATE"),
            MessageType::Broadcast => write!(f, "BROADCAST"),
        }
    }
}

/// A2A protocol message.
///
/// Messages are the core communication primitive in the FLUX A2A protocol,
/// enabling agents to communicate with each other in a distributed system.
#[derive(Debug, Clone, PartialEq)]
pub struct A2AMessage {
    /// Sender UUID (16 bytes)
    pub sender: [u8; 16],

    /// Receiver UUID (16 bytes)
    pub receiver: [u8; 16],

    /// Conversation ID UUID (16 bytes) for grouping related messages
    pub conversation_id: [u8; 16],

    /// Message type (Tell, Ask, Delegate, or Broadcast)
    pub message_type: MessageType,

    /// Message payload (variable length)
    pub payload: Vec<u8>,

    /// Trust score for the sender (0.0 to 1.0)
    pub trust_score: f32,

    /// Unix timestamp (milliseconds since epoch)
    pub timestamp: u64,
}

impl A2AMessage {
    /// Create a new A2A message.
    ///
    /// # Examples
    ///
    /// ```
    /// use flux_core::a2a::{A2AMessage, MessageType};
    ///
    /// let msg = A2AMessage::new(
    ///     [1u8; 16],
    ///     [2u8; 16],
    ///     [3u8; 16],
    ///     MessageType::Tell,
    ///     b"hello".to_vec(),
    ///     0.9,
    ///     1234567890,
    /// );
    /// ```
    pub fn new(
        sender: [u8; 16],
        receiver: [u8; 16],
        conversation_id: [u8; 16],
        message_type: MessageType,
        payload: Vec<u8>,
        trust_score: f32,
        timestamp: u64,
    ) -> Self {
        Self {
            sender,
            receiver,
            conversation_id,
            message_type,
            payload,
            trust_score,
            timestamp,
        }
    }

    /// Serialize the message to bytes.
    ///
    /// Serialization format:
    /// - sender: 16 bytes
    /// - receiver: 16 bytes
    /// - conversation_id: 16 bytes
    /// - message_type: 1 byte
    /// - payload_length: 2 bytes (u16 big-endian)
    /// - payload: N bytes
    /// - trust_score: 4 bytes (f32 big-endian)
    /// - timestamp: 8 bytes (u64 big-endian)
    ///
    /// # Examples
    ///
    /// ```
    /// use flux_core::a2a::{A2AMessage, MessageType};
    ///
    /// let msg = A2AMessage::new(
    ///     [1u8; 16],
    ///     [2u8; 16],
    ///     [3u8; 16],
    ///     MessageType::Tell,
    ///     b"hello".to_vec(),
    ///     0.9,
    ///     1234567890,
    /// );
    /// let bytes = msg.serialize();
    /// ```
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(63 + self.payload.len());

        // Fixed fields
        bytes.extend_from_slice(&self.sender);
        bytes.extend_from_slice(&self.receiver);
        bytes.extend_from_slice(&self.conversation_id);
        bytes.push(self.message_type.to_u8());

        // Payload
        let payload_len = self.payload.len() as u16;
        bytes.extend_from_slice(&payload_len.to_be_bytes());
        bytes.extend_from_slice(&self.payload);

        // Trust score and timestamp
        bytes.extend_from_slice(&self.trust_score.to_be_bytes());
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());

        bytes
    }

    /// Deserialize a message from bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes are malformed or the message type is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use flux_core::a2a::{A2AMessage, MessageType};
    ///
    /// let msg = A2AMessage::new(
    ///     [1u8; 16],
    ///     [2u8; 16],
    ///     [3u8; 16],
    ///     MessageType::Tell,
    ///     b"hello".to_vec(),
    ///     0.9,
    ///     1234567890,
    /// );
    /// let bytes = msg.serialize();
    /// let decoded = A2AMessage::deserialize(&bytes).unwrap();
    /// assert_eq!(msg, decoded);
    /// ```
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 63 {
            return Err(Error::InvalidInstruction);
        }

        let mut pos = 0;

        // Read fixed fields
        let mut sender = [0u8; 16];
        sender.copy_from_slice(&bytes[pos..pos + 16]);
        pos += 16;

        let mut receiver = [0u8; 16];
        receiver.copy_from_slice(&bytes[pos..pos + 16]);
        pos += 16;

        let mut conversation_id = [0u8; 16];
        conversation_id.copy_from_slice(&bytes[pos..pos + 16]);
        pos += 16;

        let message_type = MessageType::from_u8(bytes[pos])?;
        pos += 1;

        // Read payload
        let payload_len = u16::from_be_bytes([bytes[pos], bytes[pos + 1]]) as usize;
        pos += 2;

        if bytes.len() < pos + payload_len + 12 {
            return Err(Error::InvalidInstruction);
        }

        let payload = bytes[pos..pos + payload_len].to_vec();
        pos += payload_len;

        // Read trust score and timestamp
        let trust_score = f32::from_be_bytes([
            bytes[pos],
            bytes[pos + 1],
            bytes[pos + 2],
            bytes[pos + 3],
        ]);
        pos += 4;

        let timestamp = u64::from_be_bytes([
            bytes[pos],
            bytes[pos + 1],
            bytes[pos + 2],
            bytes[pos + 3],
            bytes[pos + 4],
            bytes[pos + 5],
            bytes[pos + 6],
            bytes[pos + 7],
        ]);

        Ok(Self {
            sender,
            receiver,
            conversation_id,
            message_type,
            payload,
            trust_score,
            timestamp,
        })
    }

    /// Create a simple tell message with minimal setup.
    ///
    /// # Examples
    ///
    /// ```
    /// use flux_core::a2a::{A2AMessage, MessageType};
    ///
    /// let msg = A2AMessage::tell([1u8; 16], [2u8; 16], b"hello world");
    /// assert_eq!(msg.message_type, MessageType::Tell);
    /// ```
    pub fn tell(sender: [u8; 16], receiver: [u8; 16], payload: &[u8]) -> Self {
        Self::new(
            sender,
            receiver,
            [0u8; 16], // Default conversation ID
            MessageType::Tell,
            payload.to_vec(),
            1.0, // Default trust score
            0,   // Default timestamp
        )
    }
}

impl fmt::Display for A2AMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "A2AMessage {{ type: {}, sender: {:?}, receiver: {:?}, payload_len: {}, trust: {:.2}, ts: {} }}",
            self.message_type,
            &self.sender[0..4],
            &self.receiver[0..4],
            self.payload.len(),
            self.trust_score,
            self.timestamp
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_roundtrip() {
        assert_eq!(MessageType::from_u8(1), Ok(MessageType::Tell));
        assert_eq!(MessageType::from_u8(2), Ok(MessageType::Ask));
        assert_eq!(MessageType::from_u8(3), Ok(MessageType::Delegate));
        assert_eq!(MessageType::from_u8(4), Ok(MessageType::Broadcast));
        assert!(MessageType::from_u8(99).is_err());
    }

    #[test]
    fn test_serialize_deserialize() {
        let original = A2AMessage::new(
            [1u8; 16],
            [2u8; 16],
            [3u8; 16],
            MessageType::Ask,
            b"test payload".to_vec(),
            0.85,
            1234567890,
        );

        let bytes = original.serialize();
        let decoded = A2AMessage::deserialize(&bytes).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_empty_payload() {
        let msg = A2AMessage::new(
            [1u8; 16],
            [2u8; 16],
            [3u8; 16],
            MessageType::Tell,
            vec![],
            1.0,
            0,
        );

        let bytes = msg.serialize();
        let decoded = A2AMessage::deserialize(&bytes).unwrap();

        assert_eq!(decoded.payload, vec![]);
    }

    #[test]
    fn test_tell_helper() {
        let msg = A2AMessage::tell([1u8; 16], [2u8; 16], b"hello");
        assert_eq!(msg.message_type, MessageType::Tell);
        assert_eq!(msg.payload, b"hello");
        assert_eq!(msg.sender, [1u8; 16]);
        assert_eq!(msg.receiver, [2u8; 16]);
    }

    #[test]
    fn test_large_payload() {
        let large_payload = vec![0xAB; 10000];
        let msg = A2AMessage::new(
            [5u8; 16],
            [6u8; 16],
            [7u8; 16],
            MessageType::Broadcast,
            large_payload.clone(),
            0.5,
            9999999999,
        );

        let bytes = msg.serialize();
        let decoded = A2AMessage::deserialize(&bytes).unwrap();

        assert_eq!(decoded.payload.len(), 10000);
        assert_eq!(decoded.payload, large_payload);
        assert_eq!(decoded.message_type, MessageType::Broadcast);
    }

    #[test]
    fn test_invalid_deserialize() {
        // Too short
        assert!(A2AMessage::deserialize(&[0, 1, 2]).is_err());

        // Invalid message type
        let mut bytes = vec![0u8; 63];
        bytes[48] = 99; // Invalid message type at position 48 (16+16+16)
        assert!(A2AMessage::deserialize(&bytes).is_err());
    }
}
