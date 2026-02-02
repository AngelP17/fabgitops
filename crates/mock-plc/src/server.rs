use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio_modbus::prelude::*;
use tokio_modbus::server::tcp::{accept_tcp_connection, Server};
use tracing::{error, info};

/// Shared state for the mock PLC
pub struct PLCState {
    pub register_value: u16,
    pub register_address: u16,
}

impl PLCState {
    pub fn new(initial_value: u16, register_address: u16) -> Self {
        Self {
            register_value: initial_value,
            register_address,
        }
    }
}

/// Start the mock Modbus TCP server
pub async fn start_server(
    bind_addr: &str,
    port: u16,
    state: Arc<Mutex<PLCState>>,
) -> anyhow::Result<()> {
    let socket_addr: SocketAddr = format!("{}:{}", bind_addr, port).parse()?;
    
    info!("Starting mock PLC server on {}", socket_addr);
    
    let listener = TcpListener::bind(socket_addr).await?;
    let server = Server::new(listener);
    
    let new_service = |_socket_addr| {
        let state = state.clone();
        Ok(Some(ModbusService { state }))
    };
    
    let on_connected = |stream, socket_addr| async move {
        accept_tcp_connection(stream, socket_addr, new_service)
    };
    
    let on_process_error = |err| {
        error!("Server error: {}", err);
    };
    
    server.serve(&on_connected, on_process_error).await?;
    
    Ok(())
}

/// Modbus service implementation
#[derive(Clone)]
struct ModbusService {
    state: Arc<Mutex<PLCState>>,
}

impl tokio_modbus::server::Service for ModbusService {
    type Request = Request<'static>;
    type Response = Response;
    type Error = std::io::Error;
    type Future = std::future::Ready<std::result::Result<Self::Response, Self::Error>>;
    
    fn call(&self, req: Self::Request) -> Self::Future {
        use tokio_modbus::bytes::Bytes;
        
        let response = match req {
            Request::ReadHoldingRegisters(addr, count) => {
                if let Ok(state) = self.state.lock() {
                    if addr == state.register_address && count == 1 {
                        Response::ReadHoldingRegisters(vec![state.register_value])
                    } else {
                        Response::Custom(0x83, Bytes::from_static(&[0x02])) // Illegal data address
                    }
                } else {
                    Response::Custom(0x83, Bytes::from_static(&[0x04])) // Server failure
                }
            }
            Request::WriteSingleRegister(addr, value) => {
                if let Ok(mut state) = self.state.lock() {
                    if addr == state.register_address {
                        state.register_value = value;
                        info!("Register {} written with value: {}", addr, value);
                        Response::WriteSingleRegister(addr, value)
                    } else {
                        Response::Custom(0x86, Bytes::from_static(&[0x02])) // Illegal data address
                    }
                } else {
                    Response::Custom(0x86, Bytes::from_static(&[0x04])) // Server failure
                }
            }
            _ => Response::Custom(0x80, Bytes::from_static(&[0x01])), // Illegal function
        };
        
        std::future::ready(Ok(response))
    }
}
