use std::net::UdpSocket;
use std::time::{Duration, Instant};

const REQUESTS: usize = 10;
const BUFFER_SIZE: usize = 64;
const MESSAGE: &[u8] = b"MESSAGE";

enum Mode {
    Sequential,
}

fn usage() -> ! {
    eprintln!("usage: client <sequential>");
    std::process::exit(1);
}

fn get_mode() -> Mode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("sequential") => Mode::Sequential,
        _ => usage(),
    }
}

fn recv_and_check(socket: &UdpSocket, buf: &mut [u8]) -> std::io::Result<()> {
    let amt = socket.recv(buf)?;

    if buf[..amt] != *MESSAGE {
        eprintln!("the message differ");
    }

    Ok(())
}

fn sequential(socket: &UdpSocket) -> std::io::Result<Duration> {
    let start_time = Instant::now();
    let mut buf = [0; BUFFER_SIZE];

    for _ in 0..REQUESTS {
        socket.send(MESSAGE)?;
        recv_and_check(socket, &mut buf)?;
    }

    Ok(start_time.elapsed())
}

fn main() -> std::io::Result<()> {
    let mode = get_mode();

    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.connect("127.0.0.1:8000")?;
    socket.set_read_timeout(Some(Duration::from_secs(1)))?;

    let elapsed = match mode {
        Mode::Sequential => sequential(&socket)?,
    };

    println!("{:?}", elapsed);

    Ok(())
}
