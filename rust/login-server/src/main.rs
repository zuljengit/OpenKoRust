use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const PACKET_START1: u8 = 0xAA;
const PACKET_START2: u8 = 0x55;
const PACKET_END1: u8 = 0x55;
const PACKET_END2: u8 = 0xAA;

const LS_VERSION_REQ: u8 = 0x01;

const LAST_VERSION: i16 = 1298;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener: TcpListener = TcpListener::bind("0.0.0.0:15100").await?;
    println!("Login server listening on 0.0.0.0:15100");

    loop {
        let (mut socket, addr): (tokio::net::TcpStream, std::net::SocketAddr) =
            listener.accept().await?;
        tokio::spawn(async move {
            println!("Client connected with IP: {addr}");
            let mut buf: Vec<u8> = vec![0; 16 * 1024];
            loop {
                let bytes_read: usize = match socket.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(_) => return,
                };
                if let Some(payload) = deframe(&buf[..bytes_read]) {
                    if let Some(reply) = handle(&payload) {
                        let framed: Vec<u8> = frame(&reply);
                        let _: Result<(), std::io::Error> =
                            socket.write_all(&framed).await;
                    }
                }
            }
        });
    }
}

fn deframe(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 6 { return None; }
    if data[0] != PACKET_START1 || data[1] != PACKET_START2 { return None; }
    let len: usize = i16::from_le_bytes([data[2], data[3]]) as usize;
    let end: usize = 4 + len;
    if data.len() < end + 2 { return None; }
    if data[end] != PACKET_END1 || data[end + 1] != PACKET_END2 { return None; }
    Some(data[4..end].to_vec())
}

fn frame(payload: &[u8]) -> Vec<u8> {
    let len: i16 = payload.len() as i16;
    let mut out: Vec<u8> = Vec::with_capacity(payload.len() + 6);
    out.push(PACKET_START1);
    out.push(PACKET_START2);
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(payload);
    out.push(PACKET_END1);
    out.push(PACKET_END2);
    out
}

fn handle(payload: &[u8]) -> Option<Vec<u8>> {
    let opcode: u8 = *payload.first()?;
    match opcode {
        LS_VERSION_REQ => {
            let mut reply: Vec<u8> = Vec::new();
            reply.push(LS_VERSION_REQ);
            reply.extend_from_slice(&LAST_VERSION.to_le_bytes());
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
    use super::{deframe, frame, handle};

    #[test]
    fn deframe_valid_packet() {
        // A valid packet: AA 55 [03 00 = length 3 as LE u16] [01 12 05 = payload] 55 AA
        let data: Vec<u8> = vec![0xAA, 0x55, 0x03, 0x00, 0x01, 0x12, 0x05, 0x55, 0xAA];
        let result: Option<Vec<u8>> = deframe(&data);
        assert_eq!(result, Some(vec![0x01, 0x12, 0x05]));
    }

    #[test]
    fn deframe_too_short() {
        let data: Vec<u8> = vec![0xAA, 0x55, 0x01];
        let result: Option<Vec<u8>> = deframe(&data);
        assert_eq!(result, None);
    }

    #[test]
    fn deframe_wrong_start_markers() {
        let data: Vec<u8> = vec![0xBB, 0x55, 0x01, 0x00, 0x01, 0x55, 0xAA];
        let result: Option<Vec<u8>> = deframe(&data);
        assert_eq!(result, None);
    }

    #[test]
    fn deframe_wrong_end_markers() {
        let data: Vec<u8> = vec![0xAA, 0x55, 0x01, 0x00, 0x01, 0x55, 0xBB];
        let result: Option<Vec<u8>> = deframe(&data);
        assert_eq!(result, None);
    }

    #[test]
    fn deframe_empty_payload() {
        let data: Vec<u8> = vec![0xAA, 0x55, 0x00, 0x00, 0x55, 0xAA];
        let result: Option<Vec<u8>> = deframe(&data);
        assert_eq!(result, Some(vec![]));
    }

    #[test]
    fn frame_valid_payload() {
        let payload: Vec<u8> = vec![0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        let result: Vec<u8> = frame(&payload);
        assert_eq!(result, vec![0xAA, 0x55, 0x05, 0x00, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x55, 0xAA]);
    }

    #[test]
    fn handle_version_request() {
        let payload: Vec<u8> = vec![0x01, 0xBB];
        let result: Option<Vec<u8>> = handle(&payload);
        assert_eq!(result, Some(vec![0x01, 0x12, 0x05]));
    }

    #[test]
    fn handle_unknown_opcode() {
        let payload: Vec<u8> = vec![0x00, 0xBB];
        let result: Option<Vec<u8>> = handle(&payload);
        assert_eq!(result, None);
    }

    #[test]
    fn handle_empty_payload() {
        let payload: Vec<u8> = vec![];
        let result: Option<Vec<u8>> = handle(&payload);
        assert_eq!(result, None);
    }

    #[test]
    fn frame_roundtrip() {
        let payload: Vec<u8> = vec![0x01, 0x12, 0x05];
        let framed: Vec<u8> = frame(&payload);
        let result: Option<Vec<u8>> = deframe(&framed);
        assert_eq!(result, Some(payload));
    }
}
