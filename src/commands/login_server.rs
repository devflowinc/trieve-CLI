// Updated example from http://rosettacode.org/wiki/Hello_world/Web_server#Rust
// to work with Rust 1.0 beta

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use tokio::sync::mpsc;

fn extract_cookie(request: &str) -> Option<String> {
    request
        .lines()
        .find(|line| line.starts_with("Cookie:"))
        .map(|line| line.trim_start_matches("Cookie: ").trim().to_string())
}

async fn handle_read(stream: &mut TcpStream, tx: mpsc::Sender<String>) {
    let mut buf = [0u8; 4096];
    match stream.read(&mut buf).await {
        Ok(_) => {
            let req_str = String::from_utf8_lossy(&buf);
            let _ = tx.send(extract_cookie(&req_str.as_ref()).unwrap()).await;
        }
        Err(e) => println!("Unable to read stream: {}", e),
    }
}

async fn handle_write(mut stream: TcpStream) {
    let response = b"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"><title>Login Success</title><link href=\"https://fonts.googleapis.com/css2?family=Roboto:wght@400;500&display=swap\" rel=\"stylesheet\"><style>body {font-family: 'Roboto', sans-serif; text-align: center; margin-top: 50px;} img {max-width: 200px;} h1, p {margin: 20px 0;}</style></head><body><img src=\"https://cdn.trieve.ai/trieve-logo.png\" alt=\"Trieve Logo\"><h1>Login Succeeded</h1><p>Return to your terminal to continue setup.</p></body></html>\r\n";
    match stream.write(response).await {
        Ok(_) => (),
        Err(e) => println!("Failed sending response: {}", e),
    }
}

async fn handle_client(mut stream: TcpStream, tx: mpsc::Sender<String>) {
    handle_read(&mut stream, tx).await;
    handle_write(stream).await;
}

pub async fn server(tx: mpsc::Sender<String>) -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8888").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await?;
        let tx = tx.clone();
        tokio::spawn(async move {
            // Process each socket concurrently.
            handle_client(socket, tx).await
        });
    }
}
