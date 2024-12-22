use embedded_recruitment_task::message::{client_message, ServerMessage};
// use log::error;
// use log::info;
use prost::Message;
use std::i32;
use std::io::Read;
use std::io::Write;
use std::{
    io,
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    time::Duration,
};

// TCP/IP Client
pub struct Client {
    ip: String,
    port: u32,
    timeout: Duration,
    stream: Option<TcpStream>,
}

impl Client {
    pub fn new(ip: &str, port: u32, timeout_ms: u64) -> Self {
        Client {
            ip: ip.to_string(),
            port,
            timeout: Duration::from_millis(timeout_ms),
            stream: None,
        }
    }

    // connect the client to the server
    pub fn connect(&mut self,id:i32) -> io::Result<()> {
        println!("client-{}:Connecting to {}:{}",id, self.ip, self.port);

        // Resolve the address
        let address = format!("{}:{}", self.ip, self.port);
        let socket_addrs: Vec<SocketAddr> = address.to_socket_addrs()?.collect();

        if socket_addrs.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid IP or port",
            ));
        }

        // Connect to the server with a timeout
        let stream = TcpStream::connect_timeout(&socket_addrs[0], self.timeout)?;
        self.stream = Some(stream);

        println!("client-{}:Connected to the server!",id);
        Ok(())
    }

    // disconnect the client
    pub fn disconnect(&mut self,id:i32) -> io::Result<()> {
        if let Some(stream) = self.stream.take() {
            stream.shutdown(std::net::Shutdown::Both)?;
        }

        println!("client-{}:Disconnected from the server!",id);
        Ok(())
    }

    // generic message to send message to the server
    pub fn send(&mut self, message: client_message::Message,id:i32) -> io::Result<()> {
        if let Some(ref mut stream) = self.stream {
            // Encode the message to a buffer
            let mut buffer = Vec::new();
            message.encode(&mut buffer);
            
            // Print the size of the buffer
            println!("client-{}: Buffer size: {} bytes", id, buffer.len());
            
            // Send the buffer to the server
            stream.write_all(&buffer)?;
            stream.flush()?;

            println!("client-{}:Sent message: {:?}",id, buffer);
            Ok(())
        } else {
            println!("msh 3aaaaaarf<=============");
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "No active connection",
            ))
        }
    }

    pub fn receive(&mut self,id:i32) -> io::Result<ServerMessage> {
        println!("Function 'receive' started.");
    
        if let Some(ref mut stream) = self.stream {
            println!("Stream is active. Attempting to read from the server...");
    
            let mut buffer = vec![0u8; 1024]; // Buffer for incoming data
            println!("CLient a3aaaa:Buffer initialized with size: {}", buffer.len());
    
            // // let bytes_read = stream.read(&mut buffer)?;
            // let bytes_read = match self.stream.read(&mut buffer) {
            //     Ok(bytes) => bytes,
            //     Err(e) => {
            //         println!("client-2: Read error");
            //         return Err(e); // Propagate or handle the error
            //     }
            // };
            // println!("Read operation completed. Bytes read: {}", bytes_read);
    
            // if bytes_read == 0 {
            //     println!("Receive function: Server disconnected.");
            //     return Err(io::Error::new(
            //         io::ErrorKind::ConnectionAborted,
            //         "Server disconnected",
            //     ));
            // }

            // Attempt to read from the stream
            let bytes_read = match stream.read(&mut buffer) {
                Ok(0) => {
                    // If read returns 0, server has disconnected
                    println!("client-{}: Server disconnected (read returned 0 bytes).",id);
                    return Err(io::Error::new(
                        io::ErrorKind::ConnectionAborted,
                        "Server disconnected",
                    ));
                }
                Ok(bytes) => {
                    // Successfully read data
                    println!("client-{}: Read operation completed. Bytes read: {}",id, bytes);
                    bytes
                }
                Err(e) => {
                    // If read fails, log the error
                    println!("client-{}: Read error: {}",id, e);
                    return Err(io::Error::new(
                        io::ErrorKind::ConnectionReset,
                        format!("Failed to read from server: {}", e),
                    ));
                }
            };
    
            // println!("Received {} bytes from the server.", bytes_read);
    
            // Decode the received message
            println!("Decoding the received message...");
            let message = ServerMessage::decode(&buffer[..bytes_read]).map_err(|e| {
                println!("Failed to decode ServerMessage: {}", e);
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to decode ServerMessage: {}", e),
                )
            })?;
    
            println!("Message decoded successfully.");
            return Ok(message);
        } else {
            println!("Receive function: No active connection.");
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "No active connection",
            ));
        }
    }
}
