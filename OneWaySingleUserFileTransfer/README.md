# Simple File Transfer Tool (FTPserver/FTPClient) User Guide

## 1. Project Overview

This is a lightweight, cross-device file transfer tool consisting of a server-side `FTPserver.exe` and a client-side `FTPClient.exe`. It enables fast file transmission via a specified IP and port, with built-in file integrity verification (hash comparison) to ensure files are transferred without corruption.

------

## 2. Tool Parameter Instructions

### 1. Server: `FTPserver.exe`

Listens for connections and receives files sent by clients.

![FTPServer.exe](https://github.com/thedarknessdied/simple_file_transport/blob/main/OneWaySingleUserFileTransfer/pic/1.png)

```cmd
Usage: FTPserver.exe [OPTIONS]

Options:
  -i, --ip <IP>         Binding listening IP address [default: 127.0.0.1]
  -p, --port <PORT>     Listening port [default: 8080]
  -h, --help            Display help information
  -V, --version         Display version information
```

### 2. Client: `FTPClient.exe`

Connects to the server and sends local files.

![Client.exe](https://github.com/thedarknessdied/simple_file_transport/blob/main/OneWaySingleUserFileTransfer/pic/3.png)

```cmd
Usage: FTPClient.exe [OPTIONS] <FILENAME>

Arguments:
  <FILENAME>            Local file path/name to send

Options:
  -i, --ip <IP>         Server IP address [default: 127.0.0.1]
  -p, --port <PORT>     Server port [default: 8080]
  -s, --save <SAVE>     (Optional) File name saved on the server
  -h, --help            Display help information
  -V, --version         Display version information
```

------

## 3. Quick Start Steps

### 1. Start the Server

In the directory containing `FTPserver.exe`, open a command prompt and run:

```cmd
# Bind to all network interface IPs and listen on port 8080 (allows LAN/public network devices to connect)
FTPserver.exe -i 0.0.0.0
```

On success, the following information will be displayed:

![Client.exe](https://github.com/thedarknessdied/simple_file_transport/blob/main/OneWaySingleUserFileTransfer/pic/2.png)

```
[+] The server has been started: 0.0.0.0:8080
[+] Maximum file name length: 255
[+] Buffer size: 8192
```

### 2. Client Sends File

In the directory containing `FTPClient.exe` and the file to send (e.g., `test.zip`), open a command prompt and run:

```cmd
# Connect to the local server and send the test.zip file
FTPClient.exe test.zip
```

The transfer process will display file details, connection status, and progress:

![Client.exe](https://github.com/thedarknessdied/simple_file_transport/blob/main/OneWaySingleUserFileTransfer/pic/4.png)

```cmd
------------------------------ FILE DETAIL ------------------------------
FileTransferInfo {
    filename: "test.zip",
    filepath: "\\?\E:\FTPClient\test.zip",
    size: 337178,
    file_type: "zip",
    hash: "8007ed88f4d8e93cb11f7ed792291750a6899b69b2812d6b28574814aa225342",
}
------------------------------ FILE DETAIL ------------------------------
[*] Trying to save as test.zip
[+] Connected to the server 127.0.0.1:8080
[*] Ready to send test.zip (337178 bytes)
[+] File sent successfully!
```

### 3. Server Completes Reception

The server will display connection information, file verification results, and a transfer completion status:

![Client.exe](https://github.com/thedarknessdied/simple_file_transport/blob/main/OneWaySingleUserFileTransfer/pic/5.png)

```
[+] New connection: 127.0.0.1:52238
[*] Prepare to receive the file: test.zip
[+] Client-side hash: 8007ed88f4d8e93cb11f7ed792291750a6899b69b2812d6b28574814aa225342
[*] File size: 337178 bytes
[*] Server-side hash: 8007ed88f4d8e93cb11f7ed792291750a6899b69b2812d6b28574814aa225342
[+] [127.0.0.1:52238] File reception completed
```

Matching hash values on the client and server confirm the file was transferred completely and without corruption.

------

## 4. Common Usage Scenarios

### Scenario 1: Cross-Device Transfer on Local Area Network (LAN)

Assume the server device IP is `192.168.1.100` and you want to transfer a file to this device:

1. Server-side execution:

   ```cmd
   FTPserver.exe -i 0.0.0.0 -p 9000
   ```

2. Client-side execution:

   ```cmd
   FTPClient.exe -i 192.168.1.100 -p 9000 "D:\test\image.png"
   ```

### Scenario 2: Custom Server-Side Saved File Name

Specify a custom file name for the server to save when sending a file:

```cmd
FTPClient.exe -s "server_test.zip" test.zip
```

------

## 5. Notes

1. **Firewall Allowlist**: If the server has a firewall enabled, ensure `FTPserver.exe` is allowed through, or manually allow the specified port (e.g., 8080).
2. **Path Permissions**: The server saves files in the directory where `FTPserver.exe` is located; ensure write permissions for this directory.
3. **File Size Limits**: The tool supports a maximum filename length of 255 characters and a buffer size of 8192 bytes; large file transfers are supported.
4. **Network Connectivity**: The client must be able to ping the server IP, and the port must not be occupied by other applications.

------

## 6. Troubleshooting

|                Issue                 |                    Troubleshooting Steps                     |
| :----------------------------------: | :----------------------------------------------------------: |
| Client cannot connect to the server  | 1. Verify the server is running and IP/port match2. Check if the firewall blocks the port3. Confirm the server IP is accessible on the LAN/public network |
| File cannot be opened after transfer | Check if client and server hash values match; retransfer if they do not match |
|           Port is occupied           | Change the server port (e.g., `FTPserver.exe -p 9000`) and update the port parameter on the client accordingly |

------

## 7. Additional Information

This tool supports basic file transfer only and lacks user authentication. It is recommended for use only in trusted network environments; avoid public exposure of the server port.

