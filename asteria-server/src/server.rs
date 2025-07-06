use anyhow::Result;
use asteria_core::{
    config::{LoadableConfig, ServerConfig},
    protocol::{Message, Packet},
};
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tracing::{debug, error, info};

use crate::input_simulator::InputSimulator;

/// TCP server that receives input events and simulates them
pub struct InputServer {
    config: ServerConfig,
    simulator: Arc<Mutex<InputSimulator>>,
}

impl InputServer {
    pub fn new() -> Result<Self> {
        let config = ServerConfig::load()?;
        let simulator = Arc::new(Mutex::new(InputSimulator::new()?));

        Ok(Self { config, simulator })
    }

    /// Start the TCP server to listen for input events
    pub async fn start(&self) -> Result<()> {
        let bind_address = format!("{}:{}", self.config.network.host, self.config.network.port);
        info!("Starting input server on {}", bind_address);

        let listener = TcpListener::bind(&bind_address).await?;
        info!("Server listening on {}", bind_address);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New client connected from {}", addr);
                    let simulator = Arc::clone(&self.simulator);

                    // Spawn a task to handle each client connection
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(stream, simulator).await {
                            error!("Error handling client {}: {}", addr, e);
                        }
                        info!("Client {} disconnected", addr);
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Handle a single client connection
    async fn handle_client(
        mut stream: TcpStream,
        simulator: Arc<Mutex<InputSimulator>>,
    ) -> Result<()> {
        let mut buffer = vec![0u8; 4096];
        let mut packet_buffer = Vec::new();

        loop {
            tokio::select! {
                // Read data from client
                result = stream.read(&mut buffer) => {
                    match result {
                        Ok(0) => {
                            debug!("Client disconnected");
                            break;
                        }
                        Ok(n) => {
                            packet_buffer.extend_from_slice(&buffer[..n]);

                            // Try to deserialize complete packets
                            while let Some(packet) = Self::try_deserialize_packet(&mut packet_buffer)? {
                                Self::process_packet(packet, &simulator).await?;
                            }
                        }
                        Err(e) => {
                            error!("Error reading from client: {}", e);
                            break;
                        }
                    }
                }

                // Handle graceful shutdown
                _ = tokio::signal::ctrl_c() => {
                    info!("Received shutdown signal");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Try to deserialize a complete packet from the buffer
    fn try_deserialize_packet(buffer: &mut Vec<u8>) -> Result<Option<Packet>> {
        if buffer.is_empty() {
            return Ok(None);
        }

        // Try to deserialize the packet
        match bincode::serde::decode_from_slice(buffer, bincode::config::standard()) {
            Ok((packet, _)) => {
                // If successful, clear the buffer and return the packet
                buffer.clear();
                Ok(Some(packet))
            }
            Err(e) => {
                // If deserialization fails, it might be incomplete data
                // For now, we'll just log and clear the buffer
                // In a production system, you'd want more sophisticated packet framing
                debug!("Failed to deserialize packet: {}", e);
                buffer.clear();
                Ok(None)
            }
        }
    }

    /// Process a received packet
    async fn process_packet(packet: Packet, simulator: &Arc<Mutex<InputSimulator>>) -> Result<()> {
        debug!("Processing packet: {}", packet.id);

        match packet.message {
            Message::InputEvent(event) => {
                let mut sim = simulator.lock().await;
                if let Err(e) = sim.simulate_input(&event) {
                    error!("Failed to simulate input event: {}", e);
                }
            }
            Message::InputEventTyped(event) => {
                let mut sim = simulator.lock().await;
                if let Err(e) = sim.simulate_typed_input(&event) {
                    error!("Failed to simulate typed input event: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Send a ping response to test connectivity
    pub async fn ping(&self, host: Option<String>) -> Result<()> {
        let target_host = host.unwrap_or(self.config.network.host.clone());
        let target_port = self.config.network.port;
        let address = format!("{}:{}", target_host, target_port);

        info!("Attempting to connect to {}", address);

        match TcpStream::connect(&address).await {
            Ok(mut stream) => {
                info!("Successfully connected to {}", address);

                // Send a simple ping packet
                let ping_packet = Packet::input_event("PING".to_string(), 0, 0);
                let serialized =
                    bincode::serde::encode_to_vec(&ping_packet, bincode::config::standard())?;
                stream.write_all(&serialized).await?;

                info!("Ping sent successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to connect to {}: {}", address, e);
                Err(e.into())
            }
        }
    }
}

impl Default for InputServer {
    fn default() -> Self {
        Self::new().expect("Failed to create input server")
    }
}
