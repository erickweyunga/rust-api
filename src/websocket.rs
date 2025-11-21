//! WebSocket protocol support (RFC 6455).
//!
//! Enable with the `websocket` feature flag.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use rust_api::{Res, WebSocketUpgrade, WebSocket, Message};
//!
//! async fn handle_ws(mut ws: WebSocket) {
//!     while let Ok(Some(msg)) = ws.receive().await {
//!         match msg {
//!             Message::Text(text) => {
//!                 ws.send_text(format!("Echo: {}", text)).await.ok();
//!             }
//!             Message::Close(_) => break,
//!             _ => {}
//!         }
//!     }
//! }
//!
//! async fn ws_route(ws: WebSocketUpgrade) -> Res {
//!     ws.upgrade(|socket| Box::pin(handle_ws(socket)))
//! }
//! ```

use bytes::{Buf, BytesMut};
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::extractors::FromRequest;
use crate::{Error, Req, Res, Result};

/// Handler function for WebSocket connections.
pub type WebSocketHandler =
    Arc<dyn Fn(WebSocket) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// WebSocket upgrade extractor.
///
/// Validates WebSocket handshake and provides upgrade method.
pub struct WebSocketUpgrade {
    key: String,
}

impl WebSocketUpgrade {
    /// Upgrade connection with handler callback.
    pub fn upgrade<F>(self, handler: F) -> Res
    where
        F: Fn(WebSocket) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static,
    {
        Res::websocket(&self.key, handler)
    }
}

#[async_trait::async_trait]
impl<S> FromRequest<S> for WebSocketUpgrade
where
    S: Send + Sync + 'static,
{
    async fn from_request(req: &mut Req, _state: &Arc<S>) -> Result<Self> {
        if !req.is_websocket_upgrade() {
            return Err(Error::Custom("Not a WebSocket upgrade request".into()));
        }

        let key = req
            .websocket_key()
            .ok_or_else(|| Error::Custom("Missing Sec-WebSocket-Key header".into()))?
            .to_string();

        Ok(WebSocketUpgrade { key })
    }
}

/// WebSocket connection over an upgraded HTTP connection.
pub struct WebSocket {
    stream: TokioIo<Upgraded>,
    buffer: BytesMut,
}

/// WebSocket message frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// UTF-8 text message.
    Text(String),
    /// Binary data message.
    Binary(Vec<u8>),
    /// Ping control frame.
    Ping(Vec<u8>),
    /// Pong control frame.
    Pong(Vec<u8>),
    /// Close control frame.
    Close(Option<CloseFrame>),
}

/// Close frame with status code and reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseFrame {
    /// Status code (e.g., 1000 for normal closure).
    pub code: u16,
    /// Optional reason text.
    pub reason: String,
}

impl WebSocket {
    pub(crate) fn new(upgraded: Upgraded) -> Self {
        Self {
            stream: TokioIo::new(upgraded),
            buffer: BytesMut::with_capacity(8192),
        }
    }

    /// Send text message.
    pub async fn send_text(&mut self, text: impl Into<String>) -> Result<()> {
        self.send(Message::Text(text.into())).await
    }

    /// Send binary message.
    pub async fn send_binary(&mut self, data: impl Into<Vec<u8>>) -> Result<()> {
        self.send(Message::Binary(data.into())).await
    }

    /// Send message.
    pub async fn send(&mut self, message: Message) -> Result<()> {
        let frame = encode_frame(&message)?;
        self.stream
            .write_all(&frame)
            .await
            .map_err(|e| Error::Custom(format!("WebSocket write error: {}", e)))?;
        Ok(())
    }

    /// Receive message.
    pub async fn receive(&mut self) -> Result<Option<Message>> {
        loop {
            if let Some(message) = decode_frame(&mut self.buffer)? {
                return Ok(Some(message));
            }

            let mut buf = vec![0u8; 4096];
            let n = self
                .stream
                .read(&mut buf)
                .await
                .map_err(|e| Error::Custom(format!("WebSocket read error: {}", e)))?;

            if n == 0 {
                return Ok(None);
            }

            self.buffer.extend_from_slice(&buf[..n]);
        }
    }

    /// Close connection.
    pub async fn close(mut self) -> Result<()> {
        self.send(Message::Close(None)).await
    }

    /// Close connection with code and reason.
    pub async fn close_with(mut self, code: u16, reason: impl Into<String>) -> Result<()> {
        self.send(Message::Close(Some(CloseFrame {
            code,
            reason: reason.into(),
        })))
        .await
    }
}

fn encode_frame(message: &Message) -> Result<Vec<u8>> {
    let (opcode, payload): (u8, Vec<u8>) = match message {
        Message::Text(text) => (0x1, text.as_bytes().to_vec()),
        Message::Binary(data) => (0x2, data.clone()),
        Message::Close(frame) => {
            let mut payload = Vec::new();
            if let Some(f) = frame {
                payload.extend_from_slice(&f.code.to_be_bytes());
                payload.extend_from_slice(f.reason.as_bytes());
            }
            (0x8, payload)
        }
        Message::Ping(data) => (0x9, data.clone()),
        Message::Pong(data) => (0xA, data.clone()),
    };

    let payload_len = payload.len();
    let mut frame = Vec::with_capacity(10 + payload_len);

    frame.push(0x80 | opcode);

    if payload_len < 126 {
        frame.push(payload_len as u8);
    } else if payload_len < 65536 {
        frame.push(126);
        frame.extend_from_slice(&(payload_len as u16).to_be_bytes());
    } else {
        frame.push(127);
        frame.extend_from_slice(&(payload_len as u64).to_be_bytes());
    }

    frame.extend_from_slice(&payload);
    Ok(frame)
}

fn decode_frame(buffer: &mut BytesMut) -> Result<Option<Message>> {
    if buffer.len() < 2 {
        return Ok(None);
    }

    let first_byte = buffer[0];
    let second_byte = buffer[1];

    let _fin = (first_byte & 0x80) != 0;
    let opcode = first_byte & 0x0F;
    let masked = (second_byte & 0x80) != 0;
    let mut payload_len = (second_byte & 0x7F) as usize;

    let mut header_len = 2;

    if payload_len == 126 {
        if buffer.len() < 4 {
            return Ok(None);
        }
        payload_len = u16::from_be_bytes([buffer[2], buffer[3]]) as usize;
        header_len = 4;
    } else if payload_len == 127 {
        if buffer.len() < 10 {
            return Ok(None);
        }
        payload_len = u64::from_be_bytes([
            buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7], buffer[8], buffer[9],
        ]) as usize;
        header_len = 10;
    }

    let mask_key_start = header_len;
    if masked {
        header_len += 4;
    }

    if buffer.len() < header_len + payload_len {
        return Ok(None);
    }

    let mut payload = buffer[header_len..header_len + payload_len].to_vec();

    if masked {
        let mask = &buffer[mask_key_start..mask_key_start + 4];
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[i % 4];
        }
    }

    buffer.advance(header_len + payload_len);

    let message = match opcode {
        0x1 => Message::Text(
            String::from_utf8(payload)
                .map_err(|_| Error::Custom("Invalid UTF-8 in text frame".into()))?,
        ),
        0x2 => Message::Binary(payload),
        0x8 => {
            let frame = if payload.len() >= 2 {
                let code = u16::from_be_bytes([payload[0], payload[1]]);
                let reason = String::from_utf8_lossy(&payload[2..]).to_string();
                Some(CloseFrame { code, reason })
            } else {
                None
            };
            Message::Close(frame)
        }
        0x9 => Message::Ping(payload),
        0xA => Message::Pong(payload),
        _ => return Err(Error::Custom(format!("Unknown opcode: {}", opcode))),
    };

    Ok(Some(message))
}
