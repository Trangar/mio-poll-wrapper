Simple wrapper around mio's [Poll](https://docs.rs/mio/latest/mio/struct.Poll.html) method.

``` rust
extern crate mio;
extern crate mio_poll_wrapper;

use mio_poll_wrapper::PollWrapper;
use mio::net::TcpListener;
use std::collections::HashMap;

fn main() {
    let mut handle = PollWrapper::new().unwrap();

    let listener = TcpListener::bind(&"0.0.0.0:8000".parse().unwrap()).unwrap();

    let process_token = handle.register(&listener).unwrap();
    let mut clients = HashMap::new();

    let result: ::std::io::Result<()> = handle.handle(|event, handle| {
        if event.token() == process_token {
            let (stream, addr) = listener.accept()?;
            println!("Accepted socket from {:?}", addr);
            let token = handle.register(&stream)?;
            clients.insert(token, stream);
        } else if let Some(client) = clients.get_mut(&event.token()) {
            println!("Received data from client {:?}", client.peer_addr());
        }
        Ok(())
    });

    if let Err(e) = result {
        println!("Could not execute: {:?}", e);
    }
}
```