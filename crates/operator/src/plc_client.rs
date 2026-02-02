use anyhow::{Context, Result};
use tokio::net::TcpStream;
use tokio_modbus::prelude::*;

/// Client for communicating with Modbus TCP devices
pub struct PLCClient {
    address: String,
    port: u16,
}

impl PLCClient {
    pub fn new(address: impl Into<String>, port: u16) -> Self {
        Self {
            address: address.into(),
            port,
        }
    }

    /// Resolve the address (supports both IPs and hostnames via DNS)
    fn addr_str(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }

    /// Read a holding register from the PLC
    pub async fn read_register(&self, register: u16) -> Result<u16> {
        let stream = TcpStream::connect(self.addr_str())
            .await
            .context("Failed to connect to PLC")?;

        let mut ctx = tcp::attach(stream);

        // Modbus registers are 0-indexed internally
        let response = ctx
            .read_holding_registers(register, 1)
            .await
            .context("Failed to read register")?;

        ctx.disconnect().await.ok();

        response.first().copied().context("Empty response from PLC")
    }

    /// Write a value to a holding register
    pub async fn write_register(&self, register: u16, value: u16) -> Result<()> {
        let stream = TcpStream::connect(self.addr_str())
            .await
            .context("Failed to connect to PLC")?;

        let mut ctx = tcp::attach(stream);

        ctx.write_single_register(register, value)
            .await
            .context("Failed to write register")?;

        ctx.disconnect().await.ok();

        Ok(())
    }

    /// Check if the PLC is reachable
    pub async fn health_check(&self) -> Result<bool> {
        match TcpStream::connect(self.addr_str()).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
