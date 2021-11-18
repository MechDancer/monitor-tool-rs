use std::time::Instant;

use crate::FigureEvent;
use async_std::{channel::Sender, net::UdpSocket, task};

pub fn spawn_background(port: u16, sender: Sender<FigureEvent>) {
    task::spawn(async move {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).await.unwrap();
        let mut buf = Box::new([0u8; 65536]);
        while let Ok((n, _)) = socket.recv_from(buf.as_mut()).await {
            let _ = sender
                .send(FigureEvent::Packet(Instant::now(), buf[..n].to_vec()))
                .await;
        }
    });
}
