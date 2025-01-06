use anyhow::Context;
use packet::RconPacket;
use rcon_minecraft::Minecraft;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

mod packet;
mod rcon_minecraft;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mc = TcpListener::bind("0.0.0.0:25575").await?;

    loop {
        let (mut socket, _) = mc.accept().await?;
        let mut minecraft = Minecraft::new();

        tokio::spawn(async move {
            let mut buf = vec![0; 8192];

            loop {
                let len = socket.read(&mut buf).await?;

                if len == 0 {
                    continue;
                }

                if let Some(rcon_packet_request) = RconPacket::deserialize(&mut buf) {
                    let rcon_packet_response = minecraft
                        .handle_rcon_packet(rcon_packet_request.clone())
                        .await
                        .context("unable to respond to rcon packet")?;

                    println!("rcon_packet_request: {:?}", rcon_packet_request);
                    println!("rcon_packet_response: {:?}", rcon_packet_response);

                    socket
                        .write_all(rcon_packet_response.serialize().as_slice())
                        .await?;
                }
            }

            Ok::<(), anyhow::Error>(())
        });
    }
}
