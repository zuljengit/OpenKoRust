use crate::config::ServerState;

pub const PACKET_START1: u8 = 0xAA;
pub const PACKET_START2: u8 = 0x55;
pub const PACKET_END1: u8 = 0x55;
pub const PACKET_END2: u8 = 0xAA;

pub const LS_VERSION_REQ: u8 = 0x01;
pub const LS_SERVER_LIST: u8 = 0xF5;
pub const LS_NEWS: u8 = 0xF6;

pub fn deframe(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 6 {
        return None;
    }
    if data[0] != PACKET_START1 || data[1] != PACKET_START2 {
        return None;
    }
    let len = i16::from_le_bytes([data[2], data[3]]) as usize;
    let end: usize = 4 + len;
    if data.len() < end + 2 {
        return None;
    }
    if data[end] != PACKET_END1 || data[end + 1] != PACKET_END2 {
        return None;
    }
    Some(data[4..end].to_vec())
}

pub fn frame(payload: &[u8]) -> Vec<u8> {
    let len = payload.len() as i16;
    let mut out: Vec<u8> = Vec::with_capacity(payload.len() + 6);
    out.push(PACKET_START1);
    out.push(PACKET_START2);
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(payload);
    out.push(PACKET_END1);
    out.push(PACKET_END2);
    out
}

pub fn write_string2(buf: &mut Vec<u8>, str: &str) {
    let len = str.len() as i16;
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(str.as_bytes());
}

pub fn handle(
    payload: &[u8],
    last_version: i16,
    servers: &[ServerState],
    news_title: &str,
    news_message: &str,
) -> Option<Vec<u8>> {
    let opcode = *payload.first()?;
    match opcode {
        LS_VERSION_REQ => {
            let mut reply: Vec<u8> = Vec::new();
            reply.push(LS_VERSION_REQ);
            reply.extend_from_slice(&last_version.to_le_bytes());
            Some(reply)
        }
        LS_SERVER_LIST => {
            let mut reply: Vec<u8> = Vec::new();
            reply.push(LS_SERVER_LIST);
            reply.push(servers.len() as u8);

            for server in servers {
                write_string2(&mut reply, &server.ip);
                write_string2(&mut reply, &server.name);

                let count = if server.user_count <= server.user_limit {
                    server.user_count
                } else {
                    -1
                };
                reply.extend_from_slice(&count.to_le_bytes());
            }

            Some(reply)
        }
        LS_NEWS => {
            let mut reply: Vec<u8> = Vec::new();
            reply.push(LS_NEWS);
            write_string2(&mut reply, news_title);
            write_string2(&mut reply, news_message);
            Some(reply)
        }
        other => {
            println!("Unhandled opcode {other:#04X}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LS_NEWS, LS_SERVER_LIST, LS_VERSION_REQ, PACKET_END1, PACKET_END2, PACKET_START1,
        PACKET_START2, ServerState, deframe, frame, handle, write_string2,
    };

    const TEST_VERSION: i16 = 1298;
    const TEST_SERVER_IP: &str = "127.0.0.1";
    const TEST_SERVER_NAME: &str = "Server 1";
    const TEST_SERVER_USER_COUNT: i16 = 0;
    const TEST_NEWS_TITLE: &str = "Test Notice";
    const TEST_NEWS_MESSAGE: &str = "Welcome!";

    #[test]
    fn deframe_valid_packet() {
        let data: Vec<u8> = vec![
            PACKET_START1,
            PACKET_START2,
            0x03,
            0x00,
            0x17,
            0x12,
            0x05,
            PACKET_END1,
            PACKET_END2,
        ];
        let result: Option<Vec<u8>> = deframe(&data);
        assert_eq!(result, Some(vec![0x17, 0x12, 0x05]));
    }

    #[test]
    fn deframe_too_short() {
        let data: Vec<u8> = vec![PACKET_START1, PACKET_START2, 0x15];
        let result: Option<Vec<u8>> = deframe(&data);
        assert!(result.is_none());
    }

    #[test]
    fn deframe_wrong_start_markers() {
        let data: Vec<u8> = vec![
            0xBB,
            PACKET_START2,
            0x01,
            0x00,
            0x15,
            PACKET_END1,
            PACKET_END2,
        ];
        let result: Option<Vec<u8>> = deframe(&data);
        assert!(result.is_none());
    }

    #[test]
    fn deframe_wrong_end_markers() {
        let data: Vec<u8> = vec![
            PACKET_START1,
            PACKET_START2,
            0x01,
            0x00,
            0x15,
            PACKET_END1,
            0xBB,
        ];
        let result: Option<Vec<u8>> = deframe(&data);
        assert!(result.is_none());
    }

    #[test]
    fn deframe_empty_payload() {
        let data: Vec<u8> = vec![
            PACKET_START1,
            PACKET_START2,
            0x00,
            0x00,
            PACKET_END1,
            PACKET_END2,
        ];
        let result: Option<Vec<u8>> = deframe(&data);
        assert_eq!(result, Some(vec![]));
    }

    #[test]
    fn frame_valid_payload() {
        let payload: Vec<u8> = vec![0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        let result: Vec<u8> = frame(&payload);
        assert_eq!(
            result,
            vec![
                PACKET_START1,
                PACKET_START2,
                0x05,
                0x00,
                0xBB,
                0xCC,
                0xDD,
                0xEE,
                0xFF,
                PACKET_END1,
                PACKET_END2,
            ]
        );
    }

    #[test]
    fn frame_roundtrip() {
        let payload: Vec<u8> = vec![0x17, 0x12, 0x05];
        let framed: Vec<u8> = frame(&payload);
        let result: Option<Vec<u8>> = deframe(&framed);
        assert_eq!(result, Some(payload));
    }

    #[test]
    fn write_string2_encodes_correctly() {
        let mut buf: Vec<u8> = Vec::new();
        write_string2(&mut buf, "Hello");
        assert_eq!(buf, vec![0x05, 0x00, 0x48, 0x65, 0x6C, 0x6C, 0x6F]);
    }

    #[test]
    fn write_string2_empty_string() {
        let mut buf: Vec<u8> = Vec::new();
        write_string2(&mut buf, "");
        assert_eq!(buf, vec![0x00, 0x00]);
    }

    #[test]
    fn handle_version_request() {
        let payload: Vec<u8> = vec![LS_VERSION_REQ, 0xBB];
        let result: Option<Vec<u8>> = handle(
            &payload,
            TEST_VERSION,
            &test_servers(),
            TEST_NEWS_TITLE,
            TEST_NEWS_MESSAGE,
        );
        assert_eq!(result, Some(vec![LS_VERSION_REQ, 0x12, 0x05]));
    }

    #[test]
    fn handle_server_list() {
        let payload: Vec<u8> = vec![LS_SERVER_LIST];
        let result: Option<Vec<u8>> = handle(
            &payload,
            TEST_VERSION,
            &test_servers(),
            TEST_NEWS_TITLE,
            TEST_NEWS_MESSAGE,
        );

        let reply: Vec<u8> = result.expect("Should return a reply");

        assert_eq!(&reply[0..2], &[LS_SERVER_LIST, 0x01]);
        assert_eq!(&reply[2..4], &[0x09, 0x00]);
        assert_eq!(&reply[4..13], TEST_SERVER_IP.as_bytes());
        assert_eq!(&reply[13..15], &[0x08, 0x00]);
        assert_eq!(&reply[15..23], TEST_SERVER_NAME.as_bytes());
        assert_eq!(&reply[23..25], TEST_SERVER_USER_COUNT.to_le_bytes());
        assert_eq!(reply.len(), 25);
    }

    #[test]
    fn handle_news() {
        let payload: Vec<u8> = vec![LS_NEWS];
        let result: Option<Vec<u8>> = handle(
            &payload,
            TEST_VERSION,
            &test_servers(),
            TEST_NEWS_TITLE,
            TEST_NEWS_MESSAGE,
        );

        let reply: Vec<u8> = result.expect("Should return a reply");

        assert_eq!(reply[0], LS_NEWS);
        assert_eq!(&reply[1..3], &[0x0B, 0x00]);
        assert_eq!(&reply[3..14], TEST_NEWS_TITLE.as_bytes());
        assert_eq!(&reply[14..16], &[0x08, 0x00]);
        assert_eq!(&reply[16..24], TEST_NEWS_MESSAGE.as_bytes());
        assert_eq!(reply.len(), 24);
    }

    #[test]
    fn handle_unknown_opcode() {
        let payload: Vec<u8> = vec![0x00, 0xBB];
        let result: Option<Vec<u8>> = handle(
            &payload,
            TEST_VERSION,
            &test_servers(),
            TEST_NEWS_TITLE,
            TEST_NEWS_MESSAGE,
        );
        assert!(result.is_none());
    }

    #[test]
    fn handle_empty_payload() {
        let payload: Vec<u8> = vec![];
        let result: Option<Vec<u8>> = handle(
            &payload,
            TEST_VERSION,
            &test_servers(),
            TEST_NEWS_TITLE,
            TEST_NEWS_MESSAGE,
        );
        assert!(result.is_none());
    }

    fn test_servers() -> Vec<ServerState> {
        vec![ServerState {
            ip: TEST_SERVER_IP.to_string(),
            name: TEST_SERVER_NAME.to_string(),
            user_count: TEST_SERVER_USER_COUNT,
            user_limit: 3000,
        }]
    }
}
