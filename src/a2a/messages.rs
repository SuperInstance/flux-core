use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
    Tell = 1,
    Ask = 2,
    Delegate = 3,
    Broadcast = 4,
}

#[derive(Debug, Clone)]
pub struct A2AMessage {
    pub sender: [u8; 16],
    pub receiver: [u8; 16],
    pub conversation_id: [u8; 16],
    pub message_type: MessageType,
    pub payload: Vec<u8>,
    pub trust_score: f32,
}

impl A2AMessage {
    pub fn new(sender: [u8; 16], receiver: [u8; 16], msg_type: MessageType, payload: Vec<u8>) -> Self {
        Self {
            sender,
            receiver,
            conversation_id: [0u8; 16],
            message_type: msg_type,
            payload,
            trust_score: 0.5,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.sender);
        buf.extend_from_slice(&self.receiver);
        buf.extend_from_slice(&self.conversation_id);
        buf.push(self.message_type as u8);
        buf.extend_from_slice(&(self.payload.len() as u16).to_le_bytes());
        buf.extend_from_slice(&self.payload);
        buf.extend_from_slice(&self.trust_score.to_le_bytes());
        buf
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 51 { return None; }
        let mut sender = [0u8; 16]; sender.copy_from_slice(&data[0..16]);
        let mut receiver = [0u8; 16]; receiver.copy_from_slice(&data[16..32]);
        let mut conv_id = [0u8; 16]; conv_id.copy_from_slice(&data[32..48]);
        let msg_type = match data[48] {
            1 => MessageType::Tell,
            2 => MessageType::Ask,
            3 => MessageType::Delegate,
            4 => MessageType::Broadcast,
            _ => return None,
        };
        let payload_len = u16::from_le_bytes([data[49], data[50]]) as usize;
        if data.len() < 51 + payload_len + 4 { return None; }
        let payload = data[51..51+payload_len].to_vec();
        let trust_score = f32::from_le_bytes([data[51+payload_len], data[52+payload_len], data[53+payload_len], data[54+payload_len]]);

        Some(Self { sender, receiver, conversation_id: conv_id, message_type: msg_type, payload, trust_score })
    }
}

impl fmt::Display for A2AMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A2A::{:?}(trust={:.2}, {} bytes)", self.message_type, self.trust_score, self.payload.len())
    }
}
