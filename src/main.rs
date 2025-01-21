use {
    anyhow::Context,
    packet::RconPacket,
    rcon_minecraft::Minecraft,
    tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    },
    tracing::{debug, error, info},
};

mod packet;
mod rcon_minecraft;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();

    let mc = TcpListener::bind("0.0.0.0:25575").await.map_err(|err| {
        error!("[acorn] error {}", err);
        err
    })?;

    info!("[acorn] listening on tcp://0.0.0.0:25575");

    loop {
        let (mut socket, addr) = mc.accept().await.map_err(|err| {
            error!("[acorn] error {}", err);
            err
        })?;
        debug!("[acorn] accepting {}", addr);
        let mut minecraft = Minecraft::new();

        tokio::spawn(async move {
            let mut buf = vec![0; 8192];

            loop {
                let len = socket.read(&mut buf).await.map_err(|err| {
                    error!("[acorn] error {}", err);
                    err
                })?;

                if len == 0 {
                    continue;
                }

                if let Some(rcon_packet_request) = RconPacket::deserialize(&mut buf) {
                    debug!(
                        "[acorn] Request {{ source: {}, id: {}, type: {:?} }}",
                        addr, rcon_packet_request.id, rcon_packet_request.r#type
                    );

                    let rcon_packet_response = minecraft
                        .handle_rcon_packet(rcon_packet_request.clone())
                        .await
                        .context("unable to respond to rcon packet")
                        .map_err(|err| {
                            error!("[acorn] error {}", err);
                            err
                        })?;

                    debug!(
                        "[acorn] Response {{ source: {}, id: {}, type: {:?} }}",
                        addr, rcon_packet_response.id, rcon_packet_response.r#type
                    );

                    socket
                        .write_all(rcon_packet_response.serialize().as_slice())
                        .await
                        .map_err(|err| {
                            error!("[acorn] error {}", err);
                            err
                        })?;
                }
            }

            #[allow(unreachable_code)]
            Ok::<(), anyhow::Error>(())
        });
    }
}
