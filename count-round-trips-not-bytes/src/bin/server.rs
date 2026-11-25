use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const BUFFER_SIZE: usize = 64;
const DELAY_MS: u64 = 50;

fn main() -> std::io::Result<()> {
    let socket = Arc::new(UdpSocket::bind("127.0.0.1:8000")?);

    let mut buf = [0; BUFFER_SIZE];
    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        let socket = Arc::clone(&socket);
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(DELAY_MS));

            if let Err(e) = socket.send_to(&buf[..amt], src) {
                eprintln!("{e}");
            }
        });
    }
}
