use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use clap::Parser;
use thiserror::Error;
use sha2::{Sha256, Digest};
use tokio::time::error::Elapsed;


#[derive(Error, Debug)]
pub enum FileTransferError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Timeout error")]
    Timeout(#[from] Elapsed),

    #[error("The length of the file name is illegal (0 or exceeds 255)")]
    InvalidFilenameLength,

    #[error("The file name is not encoded in valid UTF-8")]
    FilenameNotUtf8,

    #[error("Illegal file path, risk of path traversal")]
    InvalidPath,

    #[error("The file size is 0, which is illegal")]
    EmptyFile,

    #[error("The connection was unexpectedly disconnected, and the file transfer was not completed")]
    ConnectionAborted,

    #[error("Hash value mismatch, file transmission corrupted or tampered")]
    HashMismatch,
}

pub type Result<T> = std::result::Result<T, FileTransferError>;

const BUFFER_SIZE: usize = 8192;       
const MAX_FILENAME_LEN: usize = 255;  
const IO_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
const HASH_LENGTH: usize = 64;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1")]
    ip: String,

    #[arg(short, long, default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Args = Args::parse();

    let ip: String = args.ip;
    let port: u16 = args.port;
    let address: String = format!("{}:{}", ip, port);

    let listener: TcpListener = TcpListener::bind(&address).await?;
    println!("[+] The server has been started: {}", address);
    println!("[+] Maximum file name length: {MAX_FILENAME_LEN}");
    println!("[+] Buffer size: {BUFFER_SIZE}");

    let connection_count = Arc::new(tokio::sync::Semaphore::new(100));

    loop {
        let (socket, addr) = listener.accept().await?;
        let semaphore = Arc::clone(&connection_count);

        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            println!("\n[+] New connection: {addr}");

            match handle_client(socket).await {
                Ok(_) => println!("[+] [{addr}] File reception completed"),
                Err(e) => eprintln!("[-] [{addr}] Transmission failed: {}", e),
            }
        });
    }
}

async fn handle_client(mut socket: tokio::net::TcpStream) -> Result<()> {
    let name_len: usize = {
        let mut buf = [0; 1];
        tokio::time::timeout(IO_TIMEOUT, socket.read_exact(&mut buf)).await??;
        buf[0] as usize
    };

    if name_len == 0 || name_len > MAX_FILENAME_LEN {
        return Err(FileTransferError::InvalidFilenameLength);
    }

    let mut name_buf = vec![0; name_len];
    tokio::time::timeout(IO_TIMEOUT, socket.read_exact(&mut name_buf)).await??;

    let filename = String::from_utf8(name_buf).map_err(|_| FileTransferError::FilenameNotUtf8)?;

    let filename = Path::new(&filename)
        .file_name()
        .ok_or(FileTransferError::InvalidPath)?
        .to_string_lossy()
        .to_string();

    println!("[*] Prepare to receive the file: {}", filename);

    let mut hash_buf = [0u8; HASH_LENGTH];
    tokio::time::timeout(IO_TIMEOUT, socket.read_exact(&mut hash_buf)).await??;
    let expected_hash = String::from_utf8_lossy(&hash_buf).to_string();
    println!("[+] Client-side hash: {}", expected_hash);

    let mut size_buf = [0; 8];
    tokio::time::timeout(IO_TIMEOUT, socket.read_exact(&mut size_buf)).await??;
    
    let file_size = u64::from_le_bytes(size_buf);
    println!("[*] File size: {} bytes", file_size);
    
    if file_size == 0 {
        return Err(FileTransferError::EmptyFile);
    }

    let mut file = fs::File::create(&filename).await?;
    let mut buffer = vec![0u8; BUFFER_SIZE];
    let mut remaining = file_size;
    let mut hasher = Sha256::new();


    while remaining > 0 {
        let read_size = std::cmp::min(buffer.len(), remaining as usize);
        let n = tokio::time::timeout(IO_TIMEOUT, socket.read(&mut buffer[..read_size])).await??;

        if n == 0 {
             return Err(FileTransferError::ConnectionAborted);
        }

        file.write_all(&buffer[..n]).await?;
        hasher.update(&buffer[..n]);
        remaining -= n as u64;
    }

    file.sync_all().await?;

    let actual_hash = hex::encode(hasher.finalize());
    println!("[*] Server-side hash: {}", actual_hash);

    if actual_hash != expected_hash {
        let _ = fs::remove_file(&filename).await;
        return Err(FileTransferError::HashMismatch);
    }

    Ok(())
}