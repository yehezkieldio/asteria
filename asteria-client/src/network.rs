use anyhow::Result;
use asteria_core::{
    config::{ClientConfig, LoadableConfig},
    protocol::Packet,
};
use tokio::{
    io::{AsyncWriteExt, BufWriter},
    net::TcpStream,
    sync::mpsc,
};
use tracing::{debug, error, info, warn};

/// Network client that handles TCP communication with the server
pub struct NetworkClient {
    config: ClientConfig,
    stream: Option<BufWriter<TcpStream>>,
}

impl NetworkClient {
    pub fn new() -> Result<Self> {
        let config = ClientConfig::load()?;
        Ok(Self {
            config,
            stream: None,
        })
    }

    /// Connect to the server
    pub async fn connect(&mut self) -> Result<()> {
        let address = format!("{}:{}", "192.168.137.1", self.config.network.port);
        info!("Connecting to server at {}", address);

        let stream = TcpStream::connect(&address).await?;
        self.stream = Some(BufWriter::new(stream));

        info!("Successfully connected to server");
        Ok(())
    }

    /// Send a packet to the server
    pub async fn send_packet(&mut self, packet: Packet) -> Result<()> {
        if let Some(ref mut stream) = self.stream {
            let serialized = bincode::serde::encode_to_vec(&packet, bincode::config::standard())?;
            stream.write_all(&serialized).await?;
            stream.flush().await?;
            debug!("Sent packet: {}", packet.id);
        } else {
            warn!("Attempted to send packet without connection");
        }
        Ok(())
    }

    /// Start the network client that listens for packets from the input capture
    pub async fn start_relay(&mut self, mut packet_receiver: mpsc::Receiver<Packet>) -> Result<()> {
        self.connect().await?;

        // Handle incoming packets and relay them to the server
        while let Some(packet) = packet_receiver.recv().await {
            if let Err(e) = self.send_packet(packet).await {
                error!("Failed to send packet: {}", e);

                // Try to reconnect if the connection is lost
                if let Err(reconnect_err) = self.connect().await {
                    error!("Failed to reconnect: {}", reconnect_err);
                    // Wait before trying to reconnect
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }

        Ok(())
    }

    /// Test connectivity to the server
    pub async fn ping(&mut self) -> Result<()> {
        let address = format!("{}:{}", self.config.network.host, self.config.network.port);
        info!("Testing connectivity to {}", address);

        let stream = TcpStream::connect(&address).await?;
        let mut writer = BufWriter::new(stream);

        // Send a ping packet
        let ping_packet = Packet::input_event("PING".to_string(), 0, 0);
        let serialized = bincode::serde::encode_to_vec(&ping_packet, bincode::config::standard())?;
        writer.write_all(&serialized).await?;
        writer.flush().await?;

        info!("Ping sent successfully");
        Ok(())
    }
}

impl Default for NetworkClient {
    fn default() -> Self {
        Self::new().expect("Failed to create network client")
    }
}
