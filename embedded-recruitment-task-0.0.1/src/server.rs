use crate::message;
use log::{error, info, warn};
use prost::Message;
use std::{
    io::{self, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, OnceLock,
    },
    thread,
    time::Duration,
};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::server::thread::JoinHandle;

static CLIENT_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(())); 

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client {
            stream
            }
    }

    pub fn handle(&mut self, id: usize) -> io::Result<()> {
        println!("server-{}: Handling.....", id + 1);
        let mut buffer = [0; 512];
        println!("server-{}: Buffer initialized with size: {}", id + 1, buffer.len());
        
        /*take a lock on this mutex so that only one thread read and write on the tcp */
        let lock = CLIENT_MUTEX.lock().unwrap();
        let bytes_read = {
            match self.stream.read(&mut buffer) {
                Ok(0) => {
                    println!("server-{}: Server disconnected (read returned 0 bytes).", id + 1);
                    return Ok(());
                }
                Ok(bytes) => {
                    println!("server-{}: Read operation completed ,Bytes read: {}", id + 1, bytes);
                    bytes
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    println!("No data available yet, retrying...");
                    thread::sleep(Duration::from_millis(100));
                    return Ok(());
                }
                Err(e) => {
                    println!("server-{}: Read error: {}", id + 1, e);
                    return Err(io::Error::new(
                        io::ErrorKind::ConnectionReset,
                        format!("Failed to read from client: {}", e),
                    ));
                }
            }
        };    
        // Attempt to decode the client message
        if let Ok(client_message) = message::ClientMessage::decode(&buffer[..bytes_read]) {
            match client_message.message {
                Some(message::client_message::Message::AddRequest(add_request)) => {
                    println!("server-{}: Received AddRequest: a = {}, b = {}", id + 1, add_request.a, add_request.b);
                    
                    // Handle AddRequest
                    let result = add_request.a + add_request.b;
                    let response = message::server_message::Message::AddResponse(message::AddResponse { result });
    
                    // Encode the response into the payload
                    let mut payload = Vec::new();
                    let server_message = message::ServerMessage {
                        message: Some(response),
                    };

                    // Handle encoding error
                    if let Err(e) = server_message.encode(&mut payload) {
                        println!("server-{}: Failed to encode AddResponse message: {}", id + 1, e);
                        // Optionally, send an error message back to the client
                        return Err(io::Error::new(io::ErrorKind::Other, "Failed to encode AddResponse message"));
                    }
    
                    // Write the encoded payload to the stream
                    self.stream.write_all(&payload)?;
                    self.stream.flush()?;
                    println!("server-{}: AddResponse sent with result: {}", id + 1, result);
                }
                Some(message::client_message::Message::EchoMessage(msg)) => {
                    println!("Received EchoMessage: '{}'", msg.content);
    
                    // Encode the EchoMessage into a ServerMessage
                    let server_message = message::ServerMessage {
                        message: Some(message::server_message::Message::EchoMessage(msg)),
                    };
    
                    let mut payload = Vec::new();
                    // Handle the error when encoding the message
                    if let Err(e) = server_message.encode(&mut payload) {
                        println!("server-{}: Failed to encode EchoMessage: {}", id + 1, e);
                        return Err(io::Error::new(io::ErrorKind::Other, format!("Encoding error: {}", e)));
                    }
    
                    // Write the encoded payload to the stream
                    println!("Server-{}: Sending EchoMessage: '{}'", id + 1, payload[0]);
                    self.stream.write_all(&payload)?;
                    self.stream.flush()?;
                    println!("Server-{}: Echoed back message successfully.", id + 1);
                }
                None => {
                    println!("Server-{}: Received an empty or invalid message.",id + 1);
                }
            }
        } else {
            println!("Failed to decode in general");
        }
    
        drop(lock);
        Ok(())
    }
}

// Define the static vector
static IS_RUNNING: OnceLock<Arc<[AtomicBool; 5]>> = OnceLock::new();

pub struct Server {
    listener: TcpListener,
    client_threads: Arc<Mutex<Vec<JoinHandle<()>>>>, // Track client threads
    }

impl Server {
    // Creates a new server instance
    pub fn new(addr: &str,_id:i32) -> io::Result<Self> {
        /* 
            Initialize a static shared vector of 5 variable as we have 5 tests 
            and each test will have its own is_runing variable 
        */
        IS_RUNNING.get_or_init(|| {
            Arc::new([
                AtomicBool::new(false),
                AtomicBool::new(false),
                AtomicBool::new(false),
                AtomicBool::new(false),
                AtomicBool::new(false),
            ])
        });
        // let listener = TcpListener::bind(addr)?;
        println!("------------------------------------------------------");
        // Attempt to bind the listener
        let listener = match TcpListener::bind(addr) {
            Ok(listener) => {
                println!("The Server is Successfully bound to address: {}", addr);
                listener
            }
            Err(e) => {
                // Log different error cases
                match e.kind() {
                    ErrorKind::AddrInUse => {
                        println!("Error: The address {} is already in use.", addr);
                    }
                    ErrorKind::PermissionDenied => {
                        println!("Error: Permission denied to bind to address: {}", addr);
                    }
                    _ => {
                        println!("Error binding to address {}: {}", addr, e);
                    }
                }
                // Return the error if binding fails
                return Err(e); 
            }
        };
        /* print the address that the server is listening to */
        println!("The Server is initialized and listening on {}", addr);
        println!("------------------------------------------------------");
        Ok(Server { 
            listener ,
            client_threads: Arc::new(Mutex::new(Vec::new())), // Initialize empty thread list
        })
    }

    /* Runs the server, listening for incoming connections and handling them */
    pub fn run(&self, id: usize) -> io::Result<()> {
        /* first get a reference to the static initialized vector IS_RUNING */
        let is_running = IS_RUNNING.get().expect("Static vector not initialized");
        
        /*
            set the server to run (only the server crossponding to the passed index) 
            by setting the value to true 
        */
        is_running[id].store(true, Ordering::SeqCst); // Set the server as running
        println!("Server-{} is running on {}", id + 1, self.listener.local_addr()?);

        /* Set the listener to non-blocking mode */
        self.listener.set_nonblocking(true)?;
        println!("Server-{}: Listener set to non-blocking mode.", id + 1);

        /* 
            start runing th loop untill the is_runing variable is set to 
            false (i.e. the server is ordered to stop)
        */
        while is_running[id].load(Ordering::SeqCst) {
            /*listen to any new connection on the server */
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    println!("Server-{}: New client connected: {}", id + 1, addr);
                    /*
                        clone the variable client_thread which is responible for tracking the threads 
                        so that we can at the end make sure that all threads are joined and finished 
                    */
                    let client_threads = Arc::clone(&self.client_threads);
                    let is_running = Arc::clone(&IS_RUNNING.get().unwrap());
                    
                    /* 
                        Spawn a new thread to handle the client request as each client will be 
                        handled in an individual thread 
                    */
                    let handle = thread::spawn(move || {
                        /* create a new client and pass to it the stream  */
                        let mut client = Client::new(stream);
                        /* handle the client continously until the server is stoped */
                        while is_running[id].load(Ordering::SeqCst) {
                            if let Err(_e_) = client.handle(id) {
                                println!("Server-{}: Error handling client {}", id + 1, id + 1);
                                break;
                            } else {
                                println!("Server-{}: nothing to handle", id + 1);
                                /* if there is nothing to handle then sleep to save cpu usage*/
                                thread::sleep(Duration::from_millis(100));
                            }
                        }
                    });

                    // Save the thread handle
                    client_threads.lock().unwrap().push(handle);
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // No incoming connections, sleep briefly to reduce CPU usage
                    println!("Server-{}: No incoming connections, sleeping briefly...", id + 1);
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    error!("Server-{}: Error accepting connection: {}", id + 1, e);
                }
            }
        }
        info!("Server-{} stopped.", id + 1);
        /* stop all the threads */
        self.stop_threads();
        Ok(())
    }

    // Stops the server by setting the `is_running` flag to `false`
    pub fn stop(&self, id: usize) {
        let is_running = IS_RUNNING.get().expect("Static vector not initialized");
        if is_running[id].load(Ordering::SeqCst) {
            is_running[id].store(false, Ordering::SeqCst);
            println!("Server-{}: Shutdown signal sent.", id + 1);
        } else {
            warn!("Server-{}: Server was already stopped or not running.", id + 1);
        }
    }

    fn stop_threads(&self) {
        let mut client_threads = self.client_threads.lock().unwrap();
        while let Some(handle) = client_threads.pop() {
            if let Err(e) = handle.join() {
                eprintln!("Failed to join client thread: {:?}", e);
            }
        }
        println!("All client threads have been joined.");
    }
}