use crate::{ClientConfig, LoadableConfig, NetworkConfig, ServerConfig};

pub trait HasNetworkConfig {
    fn network(&self) -> &NetworkConfig;
}

impl HasNetworkConfig for ServerConfig {
    fn network(&self) -> &NetworkConfig {
        &self.network
    }
}

impl HasNetworkConfig for ClientConfig {
    fn network(&self) -> &NetworkConfig {
        &self.network
    }
}

pub fn init_logging<C>(config: &C)
where
    C: LoadableConfig + HasNetworkConfig,
{
    let network = config.network();
    println!("Initializing logging for {}:{}", network.host, network.port);
}
