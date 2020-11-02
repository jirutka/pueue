use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use async_std::io::{Read, Write};
use async_std::net::{TcpListener, TcpStream};
use async_std::os::unix::net::{UnixListener, UnixStream};
use async_trait::async_trait;

/// A new trait, which can be used to represent Unix- and TcpListeners.
/// This is necessary to easily write generic functions where both types can be used.
#[async_trait]
pub trait GenericListener: Sync + Send {
    async fn accept<'a>(&'a self) -> Result<Socket>;
}

#[async_trait]
impl GenericListener for TcpListener {
    async fn accept<'a>(&'a self) -> Result<Socket> {
        let (socket, _) = self.accept().await?;
        Ok(Box::new(socket))
    }
}

#[async_trait]
impl GenericListener for UnixListener {
    async fn accept<'a>(&'a self) -> Result<Socket> {
        let (socket, _) = self.accept().await?;
        Ok(Box::new(socket))
    }
}

/// A new trait, which can be used to represent Unix- and TcpStream.
/// This is necessary to easily write generic functions where both types can be used.
pub trait GenericSocket: Read + Write + Unpin + Send + Sync {}
impl GenericSocket for TcpStream {}
impl GenericSocket for UnixStream {}

/// Two convenient types, so we don't have type write Box<dyn ...> all the time.
pub type Listener = Box<dyn GenericListener>;
pub type Socket = Box<dyn GenericSocket>;

/// Get a new stream for the client.
/// This can either be a UnixStream or a TCPStream,
/// which depends on the parameters.
pub async fn get_client(unix_socket_path: Option<String>, port: Option<String>) -> Result<Socket> {
    if let Some(socket_path) = unix_socket_path {
        if !PathBuf::from(&socket_path).exists() {
            bail!(
                "Couldn't find unix socket at path {:?}. Is the daemon running yet?",
                socket_path
            );
        }
        let stream = UnixStream::connect(socket_path).await?;
        return Ok(Box::new(stream));
    }

    let port = port.unwrap();
    // Don't allow anything else than loopback until we have proper crypto
    // let address = format!("{}:{}", address, port);
    let address = format!("127.0.0.1:{}", &port);

    // Connect to socket
    let socket = TcpStream::connect(&address).await.context(format!(
        "Failed to connect to the daemon on port {}. Did you start it?",
        &port
    ))?;

    Ok(Box::new(socket))
}

/// Get a new listener for the daemon.
/// This can either be a UnixListener or a TCPlistener,
/// which depends on the parameters.
pub async fn get_listener(
    unix_socket_path: Option<String>,
    port: Option<String>,
) -> Result<Listener> {
    if let Some(socket_path) = unix_socket_path {
        // Check, if the socket already exists
        // In case it does, we have to check, if it's an active socket.
        // If it is, we have to throw an error, because another daemon is already running.
        // Otherwise, we can simply remove it.
        if PathBuf::from(&socket_path).exists() {
            if get_client(Some(socket_path.clone()), None).await.is_ok() {
                bail!(
                    "There seems to be an active pueue daemon.\n\
                      If you're sure there isn't, please remove the socket by hand \
                      inside the pueue_directory."
                );
            }

            std::fs::remove_file(&socket_path)?;
        }

        return Ok(Box::new(UnixListener::bind(socket_path).await?));
    }

    let port = port.unwrap();
    let address = format!("127.0.0.1:{}", port);
    Ok(Box::new(TcpListener::bind(address).await?))
}

fn start_ssl() {}

/// Configure the server using rusttls
/// See https://docs.rs/rustls/0.16.0/rustls/struct.ServerConfig.html for details
///
/// A TLS server needs a certificate and a fitting private key
fn load_config(options: &Options) -> io::Result<ServerConfig> {
    let certs = load_certs(&options.cert)?;
    let mut keys = load_keys(&options.key)?;

    // we don't use client authentication
    let mut config = ServerConfig::new(NoClientAuth::new());
    config
        // set this server to use one cert together with the loaded private key
        .set_single_cert(certs, keys.remove(0))
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

    Ok(config)
}

/// Load the passed certificates file
fn load_certs(path: &Path) -> io::Result<Vec<Certificate>> {
    certs(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
}

/// Load the passed keys file
fn load_keys(path: &Path) -> io::Result<Vec<PrivateKey>> {
    rsa_private_keys(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
}
