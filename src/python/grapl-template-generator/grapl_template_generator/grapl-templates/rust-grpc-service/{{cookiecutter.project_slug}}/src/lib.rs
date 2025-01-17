/*
This module just re-exports the server, client, and protos. No need to modify.
*/

pub mod server;
pub mod client;

pub mod {{cookiecutter.snake_project_name}} {
    // In the future, this will be in rust-proto.
    tonic::include_proto!("{{cookiecutter.snake_project_name}}");

    pub fn get_socket_addr() -> String {
        let env_var_name = "{{cookiecutter.snake_project_name_caps}}_GRPC_PORT";
        let port = std::env::var(env_var_name).expect(env_var_name);
        return format!("[::1]:{}", port);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
	todo!("Write some tests!")
    }
}
