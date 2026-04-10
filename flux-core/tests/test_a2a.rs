//! Integration tests for A2A protocol messages.

use flux_core::a2a::{A2AMessage, MessageType};

/// Test creating a simple A2A message
#[test]
fn test_a2a_message_creation() {
    let msg = A2AMessage::new(
        [1u8; 16],
        [2u8; 16],
        [3u8; 16],
        MessageType::Tell,
        b"hello".to_vec(),
        0.9,
        1234567890,
    );

    assert_eq!(msg.sender, [1u8; 16]);
    assert_eq!(msg.receiver, [2u8; 16]);
    assert_eq!(msg.conversation_id, [3u8; 16]);
    assert_eq!(msg.message_type, MessageType::Tell);
    assert_eq!(msg.payload, b"hello".to_vec());
    assert_eq!(msg.trust_score, 0.9);
    assert_eq!(msg.timestamp, 1234567890);
}

/// Test A2A message serialization and deserialization
#[test]
fn test_a2a_serialize_deserialize() {
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

/// Test A2A message with empty payload
#[test]
fn test_a2a_empty_payload() {
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

/// Test A2A message with large payload
#[test]
fn test_a2a_large_payload() {
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

/// Test all message types
#[test]
fn test_a2a_message_types() {
    let sender = [1u8; 16];
    let receiver = [2u8; 16];
    let conv_id = [3u8; 16];
    let payload = b"test".to_vec();

    let tell_msg = A2AMessage::new(
        sender,
        receiver,
        conv_id,
        MessageType::Tell,
        payload.clone(),
        1.0,
        0,
    );
    assert_eq!(tell_msg.message_type, MessageType::Tell);

    let ask_msg = A2AMessage::new(
        sender,
        receiver,
        conv_id,
        MessageType::Ask,
        payload.clone(),
        1.0,
        0,
    );
    assert_eq!(ask_msg.message_type, MessageType::Ask);

    let delegate_msg = A2AMessage::new(
        sender,
        receiver,
        conv_id,
        MessageType::Delegate,
        payload.clone(),
        1.0,
        0,
    );
    assert_eq!(delegate_msg.message_type, MessageType::Delegate);

    let broadcast_msg = A2AMessage::new(
        sender,
        receiver,
        conv_id,
        MessageType::Broadcast,
        payload,
        1.0,
        0,
    );
    assert_eq!(broadcast_msg.message_type, MessageType::Broadcast);
}

/// Test message type from_u8 conversion
#[test]
fn test_message_type_from_u8() {
    assert_eq!(MessageType::from_u8(1), Ok(MessageType::Tell));
    assert_eq!(MessageType::from_u8(2), Ok(MessageType::Ask));
    assert_eq!(MessageType::from_u8(3), Ok(MessageType::Delegate));
    assert_eq!(MessageType::from_u8(4), Ok(MessageType::Broadcast));
    assert!(MessageType::from_u8(99).is_err());
}

/// Test message type to_u8 conversion
#[test]
fn test_message_type_to_u8() {
    assert_eq!(MessageType::Tell.to_u8(), 1);
    assert_eq!(MessageType::Ask.to_u8(), 2);
    assert_eq!(MessageType::Delegate.to_u8(), 3);
    assert_eq!(MessageType::Broadcast.to_u8(), 4);
}

/// Test message type roundtrip
#[test]
fn test_message_type_roundtrip() {
    for ty in &[MessageType::Tell, MessageType::Ask, MessageType::Delegate, MessageType::Broadcast]
    {
        let byte = ty.to_u8();
        let decoded = MessageType::from_u8(byte).unwrap();
        assert_eq!(decoded, *ty);
    }
}

/// Test tell helper function
#[test]
fn test_tell_helper() {
    let msg = A2AMessage::tell([1u8; 16], [2u8; 16], b"hello world");

    assert_eq!(msg.message_type, MessageType::Tell);
    assert_eq!(msg.payload, b"hello world");
    assert_eq!(msg.sender, [1u8; 16]);
    assert_eq!(msg.receiver, [2u8; 16]);
    assert_eq!(msg.trust_score, 1.0);
    assert_eq!(msg.timestamp, 0);
}

/// Test serialization format
#[test]
fn test_serialization_format() {
    let msg = A2AMessage::new(
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10],
        [0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20],
        [0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30],
        MessageType::Tell,
        vec![0xAA, 0xBB, 0xCC],
        0.5,
        12345,
    );

    let bytes = msg.serialize();

    // Verify structure:
    // - 16 bytes sender
    // - 16 bytes receiver
    // - 16 bytes conversation_id
    // - 1 byte message_type
    // - 2 bytes payload_length
    // - 3 bytes payload
    // - 4 bytes trust_score
    // - 8 bytes timestamp
    // Total: 63 + 3 = 66 bytes

    assert_eq!(bytes.len(), 66);

    // Check sender UUID
    assert_eq!(bytes[0], 0x01);
    assert_eq!(bytes[15], 0x10);

    // Check receiver UUID starts at byte 16
    assert_eq!(bytes[16], 0x11);
    assert_eq!(bytes[31], 0x20);

    // Check conversation_id starts at byte 32
    assert_eq!(bytes[32], 0x21);
    assert_eq!(bytes[47], 0x30);

    // Check message_type at byte 48
    assert_eq!(bytes[48], 0x01); // MessageType::Tell

    // Check payload_length at bytes 49-50
    assert_eq!(bytes[49], 0x03);
    assert_eq!(bytes[50], 0x00);

    // Check payload at bytes 51-53
    assert_eq!(bytes[51], 0xAA);
    assert_eq!(bytes[52], 0xBB);
    assert_eq!(bytes[53], 0xCC);

    // Check trust_score at bytes 54-57 (f32)
    // 0.5 as f32 = 0x3F000000
    assert_eq!(bytes[54], 0x00);
    assert_eq!(bytes[55], 0x00);
    assert_eq!(bytes[56], 0x00);
    assert_eq!(bytes[57], 0x3F);

    // Check timestamp at bytes 58-65 (u64)
    // 12345 as u64 = 0x0000000000003039
    assert_eq!(bytes[58], 0x39);
    assert_eq!(bytes[59], 0x30);
}

/// Test invalid deserialization
#[test]
fn test_invalid_deserialize() {
    // Too short
    assert!(A2AMessage::deserialize(&[0, 1, 2]).is_err());

    // Invalid message type
    let mut bytes = vec![0u8; 63];
    bytes[48] = 99; // Invalid message type at position 48
    assert!(A2AMessage::deserialize(&bytes).is_err());

    // Truncated payload
    let mut bytes = vec![0u8; 65];
    bytes[49] = 10; // Claim 10 bytes of payload
    bytes[50] = 0;
    // But we only provide up to byte 64 (total 65 bytes)
    // 48 (header) + 2 (len) + 10 (payload) + 12 (trust+ts) = 72 bytes needed
    assert!(A2AMessage::deserialize(&bytes).is_err());
}

/// Test binary payload
#[test]
fn test_binary_payload() {
    let binary_data = vec![0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD];
    let msg = A2AMessage::new(
        [1u8; 16],
        [2u8; 16],
        [3u8; 16],
        MessageType::Tell,
        binary_data.clone(),
        0.75,
        42,
    );

    let bytes = msg.serialize();
    let decoded = A2AMessage::deserialize(&bytes).unwrap();

    assert_eq!(decoded.payload, binary_data);
}

/// Test trust_score bounds
#[test]
fn test_trust_score_values() {
    let msg1 = A2AMessage::new(
        [1u8; 16],
        [2u8; 16],
        [3u8; 16],
        MessageType::Tell,
        vec![],
        0.0,
        0,
    );

    let bytes = msg1.serialize();
    let decoded1 = A2AMessage::deserialize(&bytes).unwrap();
    assert_eq!(decoded1.trust_score, 0.0);

    let msg2 = A2AMessage::new(
        [1u8; 16],
        [2u8; 16],
        [3u8; 16],
        MessageType::Tell,
        vec![],
        1.0,
        0,
    );

    let bytes = msg2.serialize();
    let decoded2 = A2AMessage::deserialize(&bytes).unwrap();
    assert_eq!(decoded2.trust_score, 1.0);
}

/// Test timestamp handling
#[test]
fn test_timestamp() {
    let timestamp = 1617181920u64; // Some timestamp
    let msg = A2AMessage::new(
        [1u8; 16],
        [2u8; 16],
        [3u8; 16],
        MessageType::Tell,
        vec![],
        0.9,
        timestamp,
    );

    let bytes = msg.serialize();
    let decoded = A2AMessage::deserialize(&bytes).unwrap();

    assert_eq!(decoded.timestamp, timestamp);
}

/// Test message equality
#[test]
fn test_message_equality() {
    let msg1 = A2AMessage::new(
        [1u8; 16],
        [2u8; 16],
        [3u8; 16],
        MessageType::Tell,
        b"test".to_vec(),
        0.8,
        123,
    );

    let msg2 = A2AMessage::new(
        [1u8; 16],
        [2u8; 16],
        [3u8; 16],
        MessageType::Tell,
        b"test".to_vec(),
        0.8,
        123,
    );

    let msg3 = A2AMessage::new(
        [1u8; 16],
        [2u8; 16],
        [3u8; 16],
        MessageType::Ask,
        b"test".to_vec(),
        0.8,
        123,
    );

    assert_eq!(msg1, msg2);
    assert_ne!(msg1, msg3);
}

/// Test Display implementation
#[test]
fn test_message_display() {
    let msg = A2AMessage::new(
        [1u8; 16],
        [2u8; 16],
        [3u8; 16],
        MessageType::Tell,
        b"hello".to_vec(),
        0.9,
        1234567890,
    );

    let display = format!("{}", msg);

    assert!(display.contains("TELL"));
    assert!(display.contains("trust: 0.90"));
    assert!(display.contains("payload_len: 5"));
}

/// Test MessageType Display
#[test]
fn test_message_type_display() {
    assert_eq!(format!("{}", MessageType::Tell), "TELL");
    assert_eq!(format!("{}", MessageType::Ask), "ASK");
    assert_eq!(format!("{}", MessageType::Delegate), "DELEGATE");
    assert_eq!(format!("{}", MessageType::Broadcast), "BROADCAST");
}

/// Test multiple serialization roundtrips
#[test]
fn test_multiple_roundtrips() {
    let original = A2AMessage::new(
        [10u8; 16],
        [20u8; 16],
        [30u8; 16],
        MessageType::Delegate,
        b"multiple roundtrip test".to_vec(),
        0.123,
        9876543210,
    );

    // First roundtrip
    let bytes1 = original.serialize();
    let decoded1 = A2AMessage::deserialize(&bytes1).unwrap();
    assert_eq!(original, decoded1);

    // Second roundtrip
    let bytes2 = decoded1.serialize();
    let decoded2 = A2AMessage::deserialize(&bytes2).unwrap();
    assert_eq!(decoded1, decoded2);

    // Third roundtrip
    let bytes3 = decoded2.serialize();
    let decoded3 = A2AMessage::deserialize(&bytes3).unwrap();
    assert_eq!(decoded2, decoded3);
}
