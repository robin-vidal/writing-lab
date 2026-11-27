use std::cmp::min;
use std::net::UdpSocket;
use std::time::{Duration, Instant};

const REQUESTS: usize = 10;
const WINDOW_REQUESTS: usize = 200;
const BUFFER_SIZE: usize = 64;
const MESSAGE: &[u8] = b"MESSAGE";

enum Mode {
    Sequential,
    Pipelined,
    Window(usize),
}

fn usage() -> ! {
    eprintln!("usage: client <sequential|pipelined|window N>");
    std::process::exit(1);
}

fn get_mode() -> Mode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("sequential") => Mode::Sequential,
        Some("pipelined") => Mode::Pipelined,
        Some("window") => match args.get(2).and_then(|s| s.parse().ok()) {
            Some(n) => Mode::Window(n),
            None => usage(),
        },
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

fn pipelined(socket: &UdpSocket) -> std::io::Result<Duration> {
    let start_time = Instant::now();
    let mut buf = [0; BUFFER_SIZE];

    for _ in 0..REQUESTS {
        socket.send(MESSAGE)?;
    }

    for _ in 0..REQUESTS {
        recv_and_check(socket, &mut buf)?;
    }

    Ok(start_time.elapsed())
}

fn window(socket: &UdpSocket, n: usize) -> std::io::Result<Duration> {
    let start_time = Instant::now();
    let mut buf = [0; BUFFER_SIZE];

    let initial_window = min(n, WINDOW_REQUESTS);
    for _ in 0..initial_window {
        socket.send(MESSAGE)?;
    }

    let mut sent = initial_window;
    let mut in_flight = initial_window;
    while in_flight > 0 {
        recv_and_check(socket, &mut buf)?;
        if sent < WINDOW_REQUESTS {
            socket.send(MESSAGE)?;
            sent += 1;
        } else {
            in_flight -= 1;
        }
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
        Mode::Pipelined => pipelined(&socket)?,
        Mode::Window(n) => window(&socket, n)?,
    };

    println!("{:?}", elapsed);

    Ok(())
}
