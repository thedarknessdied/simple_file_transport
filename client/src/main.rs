use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use sha2::{Sha256, Digest};
use tokio::fs::File;
use clap::Parser;

#[derive(Error, Debug)]
pub enum FileTransferError {
    #[error("The file path is invalid: {0}")]
    InvalidPath(#[from] std::io::Error),
    #[error("Not a valid file")]
    InvalidFile,
    #[error("Unable to obtain the file name")]
    InvalidFilename,
    #[error("The file name is not encoded in valid UTF-8")]
    FilenameNotUtf8,
    #[error("Unknown file extension")]
    UnknownFileSuffix,
}

pub type Result<T> = std::result::Result<T, FileTransferError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferInfo {
    pub filename: String,
    pub filepath: String,
    pub size: u64,
    pub file_type: String,
    pub hash: String,
}

impl FileTransferInfo {
     pub async fn from_path(file_path: impl AsRef<Path>) -> Result<Self> {
        let path: PathBuf = file_path.as_ref().canonicalize()?;

        if !path.is_file() {
            return Err(FileTransferError::InvalidFile);
        }

        let filename = path
            .file_name()
            .ok_or(FileTransferError::InvalidFilename)?
            .to_str()
            .ok_or(FileTransferError::FilenameNotUtf8)?
            .to_string();

        let filepath = path.display().to_string();
        let metadata = tokio::fs::metadata(&path).await?;
        let size = metadata.len();
        let file_type = Self::get_file_extension(&path)?;
        let hash = Self::calculate_file_hash(&path).await?;

        Ok(Self {
            filename,
            filepath,
            size,
            file_type,
            hash,
        })
    }

    async fn calculate_file_hash(path: &Path) -> Result<String> {
        let mut file: File = File::open(path).await?;
        let mut hasher: Sha256 = Sha256::new();
        let mut buffer: [u8; 8192] = [0; 8192];

        loop {
            let n: usize = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        Ok(hex::encode(hasher.finalize()))
    }

    fn get_file_extension(path: &Path) -> Result<String> {
        Ok(path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or(FileTransferError::UnknownFileSuffix)?
            .to_lowercase()
        )
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}


#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    filename: String,

    #[arg(short, long, default_value = "127.0.0.1")]
    ip: String,

    #[arg(short, long, default_value = "8080")]
    port: u16,

    #[arg(short, long)]
    save: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();
     
    let info: FileTransferInfo = FileTransferInfo::from_path(args.filename).await?;
    println!("-------------------------------- FILE DETAIL --------------------------------");
    println!("{:#?}", info);
    println!("-------------------------------- FILE DETAIL --------------------------------");

    let save_name = args.save.unwrap_or(info.filename.clone());
    println!("[*] Trying to save as {}", save_name);
    
    let file_path: String = info.filepath;
    let mut file: File = File::open(&file_path).await?;   
    let ip: String = args.ip;
    let port: u16 = args.port;
    let address: String = format!("{}:{}", ip, port);

    let mut socket: TcpStream= TcpStream::connect(&address).await?;
    println!("[+] Connected to the server {}", address);

    let filename: String = info.filename;
    let filesize: u64 = info.size;
    let filehash: String = info.hash;

    println!("[*] Ready to send{} ({} bytes)", filename, filesize);

    socket.write_all(&[save_name.len() as u8]).await?;

    socket.write_all(save_name.as_bytes()).await?;
    
    socket.write_all(filehash.as_bytes()).await?;

    socket.write_all(&filesize.to_le_bytes()).await?;

    let mut buffer: [u8; 4096] = [0u8; 4096];
    loop {
        let n: usize = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        socket.write_all(&buffer[..n]).await?;
    }

    println!("[+] File sent successfully!");
    Ok(())
}