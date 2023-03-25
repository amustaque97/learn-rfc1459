use std::env;
use std::error::Error;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

mod server;
use server::{Errors, Server};

pub async fn parse_command(buf: &[u8]) -> Vec<String> {
    let c = String::from_utf8_lossy(&buf);
    c.strip_prefix("/")
        .unwrap()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let listener = TcpListener::bind(&addr).await?;
    println!("Server running on {}", addr);

    let mut server = Server::new();
    server.set_motd("Welcome to the default server\r\n");

    loop {
        // Async wait for an inbound socket.
        let (mut socket, addr) = listener.accept().await?;
        let mut server = server.clone();

        tokio::spawn(async move {
            // section 8.2 command parsing
            let mut buf = vec![0; 512];

            println!("Connection request from {}", addr);

            // send message of the day if present
            match server.motd {
                Some(msg) => socket
                    .write_all(msg.as_bytes())
                    .await
                    .expect("failed to send message of the day to the client"),
                None => println!("No message of the day is configured on the server"),
            }

            // In a loop, read data from the socket and write the data back
            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if n == 0 {
                    println!("No data send to the server");
                    return;
                } else {
                    println!(
                        "Request buffer {:?}",
                        String::from_utf8_lossy(&buf[0..n - 1])
                    );
                }

                let command_list = parse_command(&buf[0..n]).await;
                let command = command_list[0].as_str();
                let mut response: (Option<Errors>, String) =
                    (Some(Errors::UnknownCommand), "Unknown command".to_string());

                match command {
                    "NICK" => {
                        response = server.nick_command(command_list[1..].into(), addr).await;
                    }
                    "USER" => {
                        response = server.user_command(command_list[1..].into(), addr).await;
                    }
                    "USERS" => {
                        response = server.users_command().await;
                    }
                    "VERSION" => {
                        response = server.show_version().await;
                    }
                    "TIME" => {
                        response = server.show_time().await;
                    }
                    _ => todo!("Unknown command"),
                }

                socket
                    .write(response.1.as_bytes())
                    .await
                    .expect("failed to send response");
            }
        });
    }
}
