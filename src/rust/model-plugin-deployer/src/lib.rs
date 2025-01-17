/*
This module just re-exports the server, client, and protos. No need to modify.
*/

pub mod client;
pub mod server;

pub mod model_plugin_deployer {
    // In the future, this will be in rust-proto.
    tonic::include_proto!("graplinc.grapl.api.model_plugin_deployer.v1");
    // Everything in that package is now availabe in this namespace, e.g.
    // use crate::model_plugin_deployer::SchemaType

    const PORT_ENV_VAR: &'static str = "GRAPL_MODEL_PLUGIN_DEPLOYER_PORT";

    pub fn get_socket_addr() -> Result<std::net::SocketAddr, std::net::AddrParseError> {
        let port = std::env::var(PORT_ENV_VAR).expect(PORT_ENV_VAR);
        return format!("0.0.0.0:{}", port).parse();
    }
}
