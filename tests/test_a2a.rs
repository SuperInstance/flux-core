use flux_core::a2a::{A2AMessage, MessageType};

#[test]
fn test_a2a_roundtrip() {
    let sender = [1u8; 16];
    let receiver = [2u8; 16];
    let msg = A2AMessage::new(sender, receiver, MessageType::Tell, b"hello".to_vec());
    let bytes = msg.to_bytes();
    let decoded = A2AMessage::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.sender, sender);
    assert_eq!(decoded.receiver, receiver);
    assert_eq!(decoded.message_type, MessageType::Tell);
    assert_eq!(decoded.payload, b"hello");
}

#[test]
fn test_a2a_broadcast() {
    let msg = A2AMessage::new([0u8; 16], [0xFF; 16], MessageType::Broadcast, vec![1, 2, 3]);
    let bytes = msg.to_bytes();
    let decoded = A2AMessage::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.message_type, MessageType::Broadcast);
}
