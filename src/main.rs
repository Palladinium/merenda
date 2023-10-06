use std::{
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream},
    str,
};

use arboard::{Clipboard, GetExtLinux, SetExtLinux};
use argh::FromArgs;
use strum::{EnumString, FromRepr};

#[derive(Debug, FromArgs)]
/// A minimalistic clipboard mirroring utility
struct Args {
    /// address to connect to (or listen on)
    #[argh(option, short = 'h', default = "default_addr()")]
    address: IpAddr,

    /// port to connect to (or listen on)
    #[argh(option, short = 'p', default = "3660")]
    port: u16,

    #[argh(subcommand)]
    command: Command,
}

fn default_addr() -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
}

#[derive(Debug, FromArgs)]
#[argh(subcommand)]
enum Command {
    Server(ServerCommand),
    Set(SetCommand),
    Get(GetCommand),
}

#[derive(Debug, FromArgs)]
#[argh(subcommand)]
/// Start a server
#[argh(name = "server")]
struct ServerCommand {}

#[derive(Debug, FromArgs)]
#[argh(subcommand)]
/// read stdin and set it to the clipboard
#[argh(name = "set")]
struct SetCommand {
    /// which clipboard to set. One of: clipboard, primary, secondary
    #[argh(positional, default = "ClipboardType::default()")]
    clipboard_type: ClipboardType,
}

#[derive(Debug, FromArgs)]
#[argh(subcommand)]
/// send the contents of the clipboard to stdout
#[argh(name = "get")]
struct GetCommand {
    /// which clipboard to get. One of: clipboard, primary, secondary
    #[argh(positional, default = "ClipboardType::default()")]
    clipboard_type: ClipboardType,
}

#[derive(Debug, Default, FromRepr, EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
#[repr(u8)]
enum ClipboardType {
    Primary = 0,
    #[default]
    Clipboard = 1,
    Secondary = 2,
}

impl From<ClipboardType> for arboard::LinuxClipboardKind {
    fn from(value: ClipboardType) -> Self {
        match value {
            ClipboardType::Primary => arboard::LinuxClipboardKind::Primary,
            ClipboardType::Clipboard => arboard::LinuxClipboardKind::Clipboard,
            ClipboardType::Secondary => arboard::LinuxClipboardKind::Secondary,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid clipboard target")]
struct InvalidClipboardTarget;

fn main() {
    let args: Args = argh::from_env();

    let address = SocketAddr::new(args.address, args.port);

    match args.command {
        Command::Server(_) => run_server(address).unwrap(),
        Command::Set(set) => send_set(address, set.clipboard_type).unwrap(),
        Command::Get(get) => send_get(address, get.clipboard_type).unwrap(),
    }
}

fn run_server(address: SocketAddr) -> Result<(), ServerError> {
    let mut clipboard = Clipboard::new().unwrap_or_else(|e| panic!("Error loading clipboard: {e}"));
    let listener =
        TcpListener::bind(address).unwrap_or_else(|e| panic!("Error binding to address: {e}"));

    eprintln!("Server listening on {address}");

    loop {
        let (stream, addr) = listener
            .accept()
            .unwrap_or_else(|e| panic!("Error accepting connection: {e}"));

        eprintln!("Received connection from {addr}");
        handle_request(&mut clipboard, stream)
            .unwrap_or_else(|e| eprintln!("Error handling request: {e}"));
    }
}

#[derive(Debug, thiserror::Error)]
enum ServerError {
    #[error("IO error: {_0}")]
    Io(#[from] io::Error),
    #[error("Clipboard error: {_0}")]
    Clipboard(#[from] arboard::Error),
}

fn handle_request(clipboard: &mut Clipboard, mut stream: TcpStream) -> Result<(), RequestError> {
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf)?;

    if buf.len() < 2 {
        return Err(RequestError::Empty);
    }

    let request_type_byte = buf[0];
    let clipboard_type_byte = buf[1];

    let request_type = RequestType::from_repr(request_type_byte)
        .ok_or(RequestError::InvalidRequestType(request_type_byte))?;

    let clipboard_type = ClipboardType::from_repr(clipboard_type_byte)
        .ok_or(RequestError::InvalidClipboardType(clipboard_type_byte))?;

    eprintln!("{request_type:?} {clipboard_type:?}");

    match request_type {
        RequestType::Get => {
            let contents = clipboard.get().clipboard(clipboard_type.into()).text()?;
            stream.write_all(contents.as_bytes())?;
            stream.flush()?;
        }

        RequestType::Set => {
            let new_contents = str::from_utf8(&buf[2..])?;

            clipboard
                .set()
                .clipboard(clipboard_type.into())
                .text(new_contents)?;
        }
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum RequestError {
    #[error("IO error: {_0}")]
    Io(#[from] io::Error),
    #[error("Clipboard error: {_0}")]
    Clipboard(#[from] arboard::Error),
    #[error("Invalid UTF-8: {_0}")]
    InvalidUtf8(#[from] str::Utf8Error),
    #[error("Invalid empty request")]
    Empty,
    #[error("Invalid request type: {_0}")]
    InvalidRequestType(u8),
    #[error("Invalid clipboard type: {_0}")]
    InvalidClipboardType(u8),
}

#[derive(Debug, FromRepr)]
#[repr(u8)]
enum RequestType {
    Get = 0,
    Set = 1,
}

fn send_set(address: SocketAddr, clipboard_type: ClipboardType) -> io::Result<()> {
    let mut stream = TcpStream::connect(address)?;

    stream.write_all(&[RequestType::Set as u8, clipboard_type as u8])?;
    io::copy(&mut io::stdin(), &mut stream)?;
    stream.flush()?;
    stream.shutdown(Shutdown::Write)?;

    Ok(())
}

fn send_get(address: SocketAddr, clipboard_type: ClipboardType) -> io::Result<()> {
    let mut stream = TcpStream::connect(address)?;

    stream.write_all(&[RequestType::Get as u8, clipboard_type as u8])?;
    stream.flush()?;
    stream.shutdown(Shutdown::Write)?;

    io::copy(&mut stream, &mut io::stdout())?;

    Ok(())
}
