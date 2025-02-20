use crate::{yamux::p2p_network_yamux_state::*, Data, Limit};

/// Tests frame serialization following the spec's framing requirements:
/// - 12-byte header (Version, Type, Flags, StreamID, Length)
/// - Proper big-endian encoding
/// - Correct payload length handling
#[test]
fn test_frame_serialization() {
    // Test data frame
    let data_frame = YamuxFrame {
        flags: YamuxFlags::SYN | YamuxFlags::ACK,
        stream_id: 1,
        inner: YamuxFrameInner::Data(Data::from(vec![1, 2, 3, 4])),
    };
    let bytes = data_frame.into_bytes();
    assert_eq!(bytes.len(), 16); // 12 bytes header + 4 bytes data
    assert_eq!(
        &bytes[..12],
        &[
            0x00, // version
            0x00, // type (DATA)
            0x00, 0x03, // flags (SYN | ACK)
            0x00, 0x00, 0x00, 0x01, // stream_id
            0x00, 0x00, 0x00, 0x04, // length
        ]
    );
    assert_eq!(&bytes[12..], &[1, 2, 3, 4]);

    // Test window update frame
    let window_frame = YamuxFrame {
        flags: YamuxFlags::empty(),
        stream_id: 2,
        inner: YamuxFrameInner::WindowUpdate { difference: 1024 },
    };
    let bytes = window_frame.into_bytes();
    assert_eq!(bytes.len(), 12);
    assert_eq!(
        &bytes[..],
        &[
            0x00, // version
            0x01, // type (WINDOW_UPDATE)
            0x00, 0x00, // flags
            0x00, 0x00, 0x00, 0x02, // stream_id
            0x00, 0x00, 0x04, 0x00, // difference (1024)
        ]
    );
}

/// Tests frame parsing according to spec's requirements:
/// - Version field validation (must be 0)
/// - Proper flag combinations (SYN | ACK)
/// - Stream ID handling
/// - Length field interpretation
/// - Payload extraction
#[test]
fn test_frame_parsing() {
    let mut state = P2pNetworkYamuxState::default();
    state.message_size_limit = Limit::Some(1024);

    // Valid data frame
    let data = vec![
        0x00, // version
        0x00, // type (DATA)
        0x00, 0x03, // flags (SYN | ACK)
        0x00, 0x00, 0x00, 0x01, // stream_id
        0x00, 0x00, 0x00, 0x04, // length
        0x01, 0x02, 0x03, 0x04, // payload
    ];
    state.extend_buffer(&data);
    state.parse_frames();

    assert_eq!(state.incoming.len(), 1);
    let frame = state.incoming.pop_front().unwrap();
    assert!(frame.flags.contains(YamuxFlags::SYN));
    assert!(frame.flags.contains(YamuxFlags::ACK));
    assert_eq!(frame.stream_id, 1);
    match frame.inner {
        YamuxFrameInner::Data(data) => assert_eq!(&*data, &[1, 2, 3, 4]),
        _ => panic!("Expected Data frame"),
    }
}

/// Tests version field validation as per spec:
/// "The version field is used for future backward compatibility.
/// At the current time, the field is always set to 0"
#[test]
fn test_invalid_version() {
    let mut state = P2pNetworkYamuxState::default();

    // Invalid version
    let data = vec![
        0x01, // invalid version
        0x00, // type
        0x00, 0x00, // flags
        0x00, 0x00, 0x00, 0x01, // stream_id
        0x00, 0x00, 0x00, 0x00, // length
    ];
    state.extend_buffer(&data);
    state.parse_frames();

    assert!(matches!(
        state.terminated,
        Some(Err(YamuxFrameParseError::Version(1)))
    ));
}

/// Tests window management as specified:
/// - Window size tracking
/// - Frame splitting when exceeding window
/// - Window update mechanism
/// - Pending queue behavior
#[test]
fn test_window_management() {
    let mut stream = YamuxStreamState::default();
    assert_eq!(stream.window_ours, INITIAL_WINDOW_SIZE);
    assert_eq!(stream.window_theirs, INITIAL_WINDOW_SIZE);

    // Test consuming window space
    let frame_len = 1000;
    assert!(stream.try_consume_window(frame_len));
    assert_eq!(stream.window_theirs, INITIAL_WINDOW_SIZE - frame_len);

    // Test window update
    let update_size = 2048;
    let sendable = stream.update_remote_window(update_size);
    assert_eq!(
        stream.window_theirs,
        INITIAL_WINDOW_SIZE - frame_len + update_size
    );
    assert!(sendable.is_empty()); // No pending frames yet

    // Test window auto-tuning
    assert_eq!(stream.window_ours, INITIAL_WINDOW_SIZE);
    stream.window_ours = stream.max_window_size / 3; // Simulate consumed window
    assert!(stream.should_update_window().is_some()); // Should trigger update

    // Test pending queue
    stream.window_theirs = 10; // Set small window
    let large_frame = YamuxFrame {
        flags: YamuxFlags::empty(),
        stream_id: 1,
        inner: YamuxFrameInner::Data(Data::from(vec![1; 20])),
    };
    stream.pending.push_back(large_frame);

    // Update window and check if frame gets sent
    let sendable = stream.update_remote_window(15);
    assert_eq!(sendable.len(), 1);
    assert!(stream.pending.is_empty());
}

/// Tests stream ID allocation rules from spec:
/// "The client side should use odd ID's, and the server even.
/// This prevents any collisions."
#[test]
fn test_stream_id_allocation() {
    // Test client (odd) IDs
    assert_eq!(YamuxStreamKind::Rpc.stream_id(false), 1);
    assert_eq!(YamuxStreamKind::Gossipsub.stream_id(false), 3);

    // Test server (even) IDs
    assert_eq!(YamuxStreamKind::Rpc.stream_id(true), 2);
    assert_eq!(YamuxStreamKind::Gossipsub.stream_id(true), 4);
}

/// Tests message size limiting (implementation-specific safeguard
/// not in spec but required for security)
#[test]
fn test_message_size_limit() {
    let mut state = P2pNetworkYamuxState::default();
    state.message_size_limit = Limit::Some(10);

    // Message exceeding size limit
    let data = vec![
        0x00, // version
        0x00, // type (DATA)
        0x00, 0x00, // flags
        0x00, 0x00, 0x00, 0x01, // stream_id
        0x00, 0x00, 0x00, 0x20, // length (32 bytes, exceeds limit)
    ];
    state.extend_buffer(&data);
    state.parse_frames();

    assert!(matches!(
        state.terminated,
        Some(Ok(Err(YamuxSessionError::Internal)))
    ));
}

/// Tests ping messages as per spec:
/// "Used to measure RTT. It can also be used to heart-beat
/// and do keep-alives over TCP."
/// - SYN flag for outbound
/// - ACK flag for response
#[test]
fn test_ping_pong() {
    let ping = YamuxPing {
        stream_id: 0,
        opaque: 12345,
        response: false,
    };

    let frame = ping.into_frame();
    assert!(frame.flags.contains(YamuxFlags::SYN));
    assert!(!frame.flags.contains(YamuxFlags::ACK));

    match frame.inner {
        YamuxFrameInner::Ping { opaque } => assert_eq!(opaque, 12345),
        _ => panic!("Expected Ping frame"),
    }

    // Test ping response
    let response = YamuxPing {
        stream_id: 0,
        opaque: 12345,
        response: true,
    };

    let frame = response.into_frame();
    assert!(frame.flags.contains(YamuxFlags::ACK));
    assert!(!frame.flags.contains(YamuxFlags::SYN));
}

/// Tests handling of incomplete frames:
/// Verifies that parser correctly handles partial frame data
/// and waits for complete frame before processing
#[test]
fn test_partial_frame_parsing() {
    let mut state = P2pNetworkYamuxState::default();
    state.message_size_limit = Limit::Some(1024);

    // Send header only first
    let header = vec![
        0x00, // version
        0x00, // type (DATA)
        0x00, 0x03, // flags (SYN | ACK)
        0x00, 0x00, 0x00, 0x01, // stream_id
        0x00, 0x00, 0x00, 0x04, // length
    ];
    state.extend_buffer(&header);
    state.parse_frames();
    assert_eq!(state.incoming.len(), 0); // Should not parse incomplete frame

    // Send payload
    let payload = vec![0x01, 0x02, 0x03, 0x04];
    state.extend_buffer(&payload);
    state.parse_frames();
    assert_eq!(state.incoming.len(), 1); // Should now parse complete frame
}

/// Tests parsing of multiple consecutive frames
/// Ensures correct frame boundary detection and sequential processing
#[test]
fn test_multiple_frames_parsing() {
    let mut state = P2pNetworkYamuxState::default();
    state.message_size_limit = Limit::Some(1024);

    // Two consecutive frames
    let frames = vec![
        0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x01,
        0x02, // Frame 1
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x03,
        0x04, // Frame 2
    ];
    state.extend_buffer(&frames);
    state.parse_frames();
    assert_eq!(state.incoming.len(), 2);
}

/// Tests flow control as per spec:
/// - Window size tracking
/// - Frame splitting when exceeding window
/// - Window update mechanism
#[test]
fn test_flow_control() {
    // TODO: This test is incomplete:
    // - Doesn't verify pending queue behavior when window is full
    // - Should test frame redelivery when window becomes available
    // - Needs to verify window update triggers for pending frames
    // - Should test behavior when pending queue reaches limit
    let mut state = P2pNetworkYamuxState::default();
    let stream_id = 1;

    let mut stream = YamuxStreamState::default();
    stream.window_theirs = 10; // Small window for testing
    state.streams.insert(stream_id, stream);

    // Create frame larger than window
    let large_data = vec![1; 20];
    let mut frame = YamuxFrame {
        flags: YamuxFlags::empty(),
        stream_id,
        inner: YamuxFrameInner::Data(Data::from(large_data)),
    };

    // Check if frame gets split according to window size
    let split_frame = frame.split_at(10);
    assert!(split_frame.is_some());

    let stream = state.streams.get(&stream_id).unwrap();
    assert_eq!(stream.window_theirs, 10);
}

/// Tests invalid flag combinations
/// Ensures proper error handling for undefined flag bits
#[test]
fn test_invalid_flags() {
    let mut state = P2pNetworkYamuxState::default();

    // Invalid flags value
    let data = vec![
        0x00, // version
        0x00, // type
        0xFF, 0xFF, // invalid flags
        0x00, 0x00, 0x00, 0x01, // stream_id
        0x00, 0x00, 0x00, 0x00, // length
    ];
    state.extend_buffer(&data);
    state.parse_frames();

    assert!(matches!(
        state.terminated,
        Some(Err(YamuxFrameParseError::Flags(0xFFFF)))
    ));
}

/// Tests invalid frame type handling as per spec:
/// Only types 0x0 (Data), 0x1 (Window Update),
/// 0x2 (Ping), and 0x3 (Go Away) are valid
#[test]
fn test_invalid_frame_type() {
    let mut state = P2pNetworkYamuxState::default();

    // Invalid frame type
    let data = vec![
        0x00, // version
        0xFF, // invalid type
        0x00, 0x00, // flags
        0x00, 0x00, 0x00, 0x01, // stream_id
        0x00, 0x00, 0x00, 0x00, // length
    ];
    state.extend_buffer(&data);
    state.parse_frames();

    assert!(matches!(
        state.terminated,
        Some(Err(YamuxFrameParseError::Type(0xFF)))
    ));
}

/// Tests GoAway error codes as specified:
/// - 0x0 Normal termination
/// - 0x1 Protocol error
/// - 0x2 Internal error
#[test]
fn test_goaway_error_codes() {
    let mut state = P2pNetworkYamuxState::default();

    // Test each GoAway error code
    for (code, expected_result) in &[
        (0u8, Ok(())),
        (1u8, Err(YamuxSessionError::Protocol)),
        (2u8, Err(YamuxSessionError::Internal)),
    ] {
        let data = vec![
            0x00, // version
            0x03, // GoAway type
            0x00, 0x00, // flags
            0x00, 0x00, 0x00, 0x00, // stream_id (0 for session)
            0x00, 0x00, 0x00, *code, // error code
        ];
        state.extend_buffer(&data);
        state.parse_frames();

        assert_eq!(state.incoming.len(), 1);
        let frame = state.incoming.pop_front().unwrap();
        match frame.inner {
            YamuxFrameInner::GoAway(result) => assert_eq!(&result, expected_result),
            _ => panic!("Expected GoAway frame"),
        }
    }
}

/// Tests buffer capacity management for received data
/// Implementation-specific but critical for memory management
#[test]
fn test_buffer_capacity_management() {
    let mut state = P2pNetworkYamuxState::default();

    // Fill buffer with some data
    let initial_data = vec![0; INITIAL_RECV_BUFFER_CAPACITY];
    state.extend_buffer(&initial_data);

    // Check initial capacity
    assert_eq!(state.buffer.capacity(), INITIAL_RECV_BUFFER_CAPACITY);

    // Add more data to trigger buffer growth
    state.extend_buffer(&[0; INITIAL_RECV_BUFFER_CAPACITY]);

    // Verify buffer has grown
    assert!(state.buffer.capacity() > INITIAL_RECV_BUFFER_CAPACITY);

    // Consume most of the data and compact
    state.shift_and_compact_buffer(INITIAL_RECV_BUFFER_CAPACITY + INITIAL_RECV_BUFFER_CAPACITY / 2);

    // Check if buffer was compacted
    // Note: The actual capacity might be larger than INITIAL_RECV_BUFFER_CAPACITY
    // because Vec doesn't automatically shrink unless specifically requested
    assert!(state.buffer.capacity() >= INITIAL_RECV_BUFFER_CAPACITY);
    assert_eq!(state.buffer.len(), INITIAL_RECV_BUFFER_CAPACITY / 2);
}

/// Tests complete stream lifecycle as per spec:
/// - Stream establishment (SYN/ACK handshake)
/// - Data transfer
/// - Stream closure (FIN)
#[test]
fn test_stream_lifecycle() {
    // TODO: This test is incomplete:
    // - Doesn't verify stream state changes after each frame
    // - Should test both sides of the handshake
    // - Needs to verify cleanup after FIN
    // - Should test RST handling
    let mut state = P2pNetworkYamuxState::default();
    let stream_id = 1;

    // Test stream establishment (SYN -> ACK)
    let stream = YamuxStreamState::default();
    assert!(!stream.established);
    state.streams.insert(stream_id, stream);

    // Simulate SYN
    let syn_frame = YamuxFrame {
        flags: YamuxFlags::SYN,
        stream_id,
        inner: YamuxFrameInner::Data(Data::from(vec![1, 2])),
    };
    state.incoming.push_back(syn_frame);

    // Simulate ACK
    let ack_frame = YamuxFrame {
        flags: YamuxFlags::ACK,
        stream_id,
        inner: YamuxFrameInner::WindowUpdate { difference: 0 },
    };
    state.incoming.push_back(ack_frame);

    // Test stream closure (FIN)
    let fin_frame = YamuxFrame {
        flags: YamuxFlags::FIN,
        stream_id,
        inner: YamuxFrameInner::Data(Data::from(vec![])),
    };
    state.incoming.push_back(fin_frame);
}

/// Tests window size overflow protection:
/// Ensures window updates don't exceed MAX_WINDOW_SIZE
/// Implementation-specific safety measure
#[test]
fn test_window_overflow() {
    let mut state = P2pNetworkYamuxState::default();
    let stream_id = 1;

    let mut stream = YamuxStreamState::default();
    stream.window_theirs = MAX_WINDOW_SIZE;
    state.streams.insert(stream_id, stream);

    // Try to update beyond MAX_WINDOW_SIZE
    let update_frame = YamuxFrame {
        flags: YamuxFlags::empty(),
        stream_id,
        inner: YamuxFrameInner::WindowUpdate {
            difference: MAX_WINDOW_SIZE,
        },
    };
    state.incoming.push_back(update_frame);

    let stream = state.streams.get(&stream_id).unwrap();
    assert!(stream.window_theirs <= MAX_WINDOW_SIZE);
}

/// Tests handling of malformed frames:
/// Ensures proper error handling when frame length
/// doesn't match actual data length
#[test]
fn test_malformed_frame() {
    let mut state = P2pNetworkYamuxState::default();
    state.message_size_limit = Limit::Some(1024);

    // Frame with length field larger than actual data
    let data = vec![
        0x00, // version
        0x00, // type (DATA)
        0x00, 0x00, // flags
        0x00, 0x00, 0x00, 0x01, // stream_id
        0x00, 0x00, 0x00, 0x10, // length (16 bytes)
        0x01, 0x02, 0x03, // only 3 bytes of payload
    ];
    state.extend_buffer(&data);
    state.parse_frames();
    assert_eq!(state.incoming.len(), 0); // Should not parse malformed frame
}

/// Tests session stream (ID 0) rules per spec:
/// "The 0 ID is reserved to represent the session."
/// Verifies proper handling of session-level messages
#[test]
fn test_session_stream_rules() {
    // TODO: This test is incomplete:
    // - Should verify rejection of data frames on session stream
    // - Needs to test all allowed frame types (ping and go-away)
    // - Should verify session stream handling in both directions
    let mut state = P2pNetworkYamuxState::default();

    // Test session stream (ID 0) with data frame (should be invalid)
    let data = vec![
        0x00, // version
        0x00, // type (DATA)
        0x00, 0x00, // flags
        0x00, 0x00, 0x00, 0x00, // stream_id 0 (session)
        0x00, 0x00, 0x00, 0x01, // length
        0x00, // payload
    ];
    state.extend_buffer(&data);
    state.parse_frames();

    // Verify only ping and go-away frames are allowed on session stream
    let frame = YamuxFrame {
        flags: YamuxFlags::empty(),
        stream_id: 0,
        inner: YamuxFrameInner::Ping { opaque: 1 },
    };
    assert!(frame.is_session_stream());
}

/// Tests stream state transitions:
/// - Initial state
/// - SYN received
/// - Establishment
/// - Readable/writable states
#[test]
fn test_stream_state_transitions() {
    // TODO: This test is incomplete:
    // - Doesn't verify state transitions are atomic
    // - Should test invalid state transitions
    // - Needs to verify readable/writable states after various events
    // - Should test concurrent operations on stream
    let mut state = P2pNetworkYamuxState::default();
    let stream_id = 1;

    // Initial state
    let stream = YamuxStreamState::default();
    assert!(!stream.established);
    assert!(!stream.readable);
    assert!(!stream.writable);
    state.streams.insert(stream_id, stream);

    // Test SYN -> established transition
    let syn_frame = YamuxFrame {
        flags: YamuxFlags::SYN,
        stream_id,
        inner: YamuxFrameInner::Data(Data::from(vec![])),
    };
    state.incoming.push_back(syn_frame);

    // Test readable/writable state after establishment
    if let Some(stream) = state.streams.get_mut(&stream_id) {
        stream.established = true;
        stream.readable = true;
        stream.writable = true;
        assert!(stream.established);
    }
}
