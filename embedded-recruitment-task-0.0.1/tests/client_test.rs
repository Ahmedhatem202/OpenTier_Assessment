use embedded_recruitment_task::{
    message::{client_message, server_message, AddRequest, EchoMessage},
    server::Server
};
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};
use once_cell::sync::Lazy;
mod client;

/* A static server that will be shared by all tests instead of a server for each test*/
static SERVER: Lazy<Arc<Server>> = Lazy::new(|| {
    println!("SERVER.clone is called only one time even if any test call it");
    Arc::new(Server::new("localhost:8080",1).expect("Failed to start server"))
});

// Helper function to set up the server thread (it will only run once)
fn setup_server_thread(id:usize) -> JoinHandle<()> {
    println!("setup_server_thread is called from test number {}",id+1);
    let server = SERVER.clone();
    thread::spawn(move || {
        // Server running on a separate thread
        server.run(id).unwrap();
    })
}

#[test]
fn test_client_connection() {
    // Set up the server in a separate thread
    let server = SERVER.clone();
    let handle = setup_server_thread(0);

    // Create and connect the client
    let mut client = client::Client::new("localhost", 8080, 1000);
    assert!(client.connect(1).is_ok(), "Failed to connect to the server");

    // Disconnect the client
    assert!(
        client.disconnect(1).is_ok(),
        "Failed to disconnect from the server"
    );
    
    // Stop the server and wait for thread to finish
    server.stop(0);
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

#[test]
fn test_client_echo_message() {
    // Set up the server in a separate thread
    let server = SERVER.clone();
    let handle = setup_server_thread(1);

    // Create and connect the client
    let mut client = client::Client::new("localhost", 8080, 2000);
    assert!(client.connect(2).is_ok(), "Failed to connect to the server");

    // Prepare the message
    let mut echo_message = EchoMessage::default();
    echo_message.content = "Hello, World!".to_string();
    let message = client_message::Message::EchoMessage(echo_message.clone());

    // Lock the mutex before calling send
    // let lock = CLIENT_MUTEX.lock().unwrap();
    assert!(client.send(message,2).is_ok(), "Failed to send message");

    let response = client.receive(2);
    assert!(
        response.is_ok(),
        "client-2: Failed to receive response for EchoMessage: {:?}",
        response.unwrap_err() // Log the error
    );

    match response.unwrap().message {
        Some(server_message::Message::EchoMessage(echo)) => {
            assert_eq!(
                echo.content, echo_message.content,
                "Echoed message content does not match"
            );
        }
        _ => panic!("Expected EchoMessage, but received a different message"),
    }

    // Disconnect the client
    assert!(
        client.disconnect(2).is_ok(),
        "Failed to disconnect from the server"
    );
    
    // Stop the server and wait for thread to finish
    server.stop(1);
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

#[test]
fn test_multiple_echo_messages() {
    // Set up the server in a separate thread
    let server = SERVER.clone();
    let handle = setup_server_thread(2);

    // Create and connect the client
    let mut client = client::Client::new("localhost", 8080, 1000);
    assert!(client.connect(3).is_ok(), "Failed to connect to the server");

    // Prepare multiple messages
    let messages = vec![
        "Hello, World!".to_string(),
        "How are you?".to_string(),
        "Goodbye!".to_string(),
    ];

    // Send and receive multiple messages
    for message_content in messages {
        let mut echo_message = EchoMessage::default();
        echo_message.content = message_content.clone();
        let message = client_message::Message::EchoMessage(echo_message);

        assert!(client.send(message,3).is_ok(), "Failed to send message");

        let response = client.receive(3);
        assert!(
            response.is_ok(),
            "Failed to receive response for EchoMessage"
        );

        match response.unwrap().message {
            Some(server_message::Message::EchoMessage(echo)) => {
                assert_eq!(
                    echo.content, message_content,
                    "Echoed message content does not match"
                );
            }
            _ => panic!("Expected EchoMessage, but received a different message"),
        }
    }

    // Disconnect the client
    assert!(
        client.disconnect(3).is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server and wait for thread to finish
    server.stop(2);
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

#[test]
fn test_multiple_clients() {
    // Set up the server in a separate thread
    let handle = setup_server_thread(3);
    let server = SERVER.clone();

    // Create and connect multiple clients
    let mut clients = vec![
        client::Client::new("localhost", 8080, 1000),
        client::Client::new("localhost", 8080, 1000),
        client::Client::new("localhost", 8080, 1000),
    ];

    for client in clients.iter_mut() {
        assert!(client.connect(4).is_ok(), "Failed to connect to the server");
    }

    // Prepare multiple messages
    let messages = vec![
        "Hello, World!".to_string(),
        "How are you?".to_string(),
        "Goodbye!".to_string(),
    ];

    // Send and receive multiple messages for each client
    for message_content in messages {
        let mut echo_message = EchoMessage::default();
        echo_message.content = message_content.clone();
        let message = client_message::Message::EchoMessage(echo_message.clone());

        for client in clients.iter_mut() {
            assert!(
                client.send(message.clone(),4).is_ok(),
                "Failed to send message"
            );

            // Receive the echoed message
            let response = client.receive(4);
            assert!(
                response.is_ok(),
                "Failed to receive response for EchoMessage"
            );

            match response.unwrap().message {
                Some(server_message::Message::EchoMessage(echo)) => {
                    assert_eq!(
                        echo.content, message_content,
                        "Echoed message content does not match"
                    );
                }
                _ => panic!("Expected EchoMessage, but received a different message"),
            }
        }
    }

    // Disconnect the clients
    for client in clients.iter_mut() {
        assert!(
            client.disconnect(4).is_ok(),
            "Failed to disconnect from the server"
        );
    }

    // Stop the server and wait for thread to finish
    server.stop(3);
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}
#[test]
fn test_client_add_request() {
    // Set up the server in a separate thread
    let handle = setup_server_thread(4);
    let server = SERVER.clone();

    // Create and connect the client
    let mut client = client::Client::new("localhost", 8080, 1000);
    assert!(client.connect(5).is_ok(), "Failed to connect to the server");

    // Prepare the message
    let mut add_request = AddRequest::default();
    add_request.a = 10;
    add_request.b = 20;
    let message = client_message::Message::AddRequest(add_request.clone());

    // Send the message to the server
    assert!(client.send(message, 5).is_ok(), "Failed to send message");
    
    // Receive the response
    let response = {
        client.receive(5)
    };
    assert!(response.is_ok(), "Failed to receive response for AddRequest");
    
    match response.unwrap().message {
        Some(server_message::Message::AddResponse(add_response)) => {
            assert_eq!(
                add_response.result,
                add_request.a + add_request.b,
                "AddResponse result does not match"
            );
        }
        _ => panic!("Expected AddResponse, but received a different message"),
    }
    // Disconnect the client
    assert!(client.disconnect(5).is_ok(), "Failed to disconnect from the server");
    
    // Stop the server and wait for the thread to finish
    server.stop(4);
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}
