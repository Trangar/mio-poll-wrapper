#![deny(missing_docs)]

//! Simple wrapper around mio's [Poll](https://docs.rs/mio/latest/mio/struct.Poll.html) method.
//!
//! ``` rust,no_run
//! use mio_poll_wrapper::{PollWrapper, Handle};
//! use mio::net::TcpListener;
//! use std::collections::HashMap;
//!
//! let mut handle = PollWrapper::new().unwrap();
//!
//! let mut listener = TcpListener::bind("0.0.0.0:8000".parse().unwrap()).unwrap();
//!
//! let process_token = handle.register(&mut listener).unwrap();
//! let mut clients = HashMap::new();
//!
//! let result: ::std::io::Result<()> = handle.handle(|event, handle| {
//!     if event.token() == process_token {
//!         let (mut stream, addr) = listener.accept()?;
//!         println!("Accepted socket from {:?}", addr);
//!         let token = handle.register(&mut stream)?;
//!         clients.insert(token, stream);
//!     } else if let Some(client) = clients.get_mut(&event.token()) {
//!         println!("Received data from client {:?}", client.peer_addr());
//!     }
//!     Ok(())
//! });
//!
//! if let Err(e) = result {
//!     println!("Could not execute: {:?}", e);
//! }
//! ```

extern crate mio;

use mio::{
    event::{Event, Source},
    Events, Interest, Poll, Token,
};

/// A wrapper around mio's Poll method
///
/// You can create this
pub struct PollWrapper {
    poll: Poll,
    tokens: Vec<Token>,
    next_token_id: usize,
}

impl PollWrapper {
    /// Create a new poll wrapper
    pub fn new() -> ::std::io::Result<PollWrapper> {
        Ok(PollWrapper {
            poll: Poll::new()?,
            tokens: Vec::new(),
            next_token_id: 0,
        })
    }

    /// Start the poll routine. Every time an event gets received, the callback handler gets called.
    ///
    /// The first argument of the handler is the event that is received.
    ///
    /// The second argument is a handle. See [Handle] for more information.
    pub fn handle<E>(
        mut self,
        mut handler: impl FnMut(&Event, &mut dyn Handle) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut events = Events::with_capacity(10);
        loop {
            self.poll.poll(&mut events, None).unwrap();
            for event in &events {
                let mut handle = PollHandle {
                    poll: &self.poll,
                    tokens: Vec::new(),
                    next_token_id: &mut self.next_token_id,
                };
                handler(event, &mut handle)?;
                for token in handle.tokens {
                    self.tokens.push(token);
                }
            }
        }
    }
}

/// A handle that gets passed to the callback method of [PollWrapper].
///
/// This handle allows you to register evented methods to the poll while the wrapper is running.
pub struct PollHandle<'a> {
    poll: &'a Poll,
    tokens: Vec<Token>,
    next_token_id: &'a mut usize,
}

/// A generic trait of a handle that can register evented systems with a poll obj
pub trait Handle {
    /// Register an evented with the poll.
    /// This returns the token that was registered.
    fn register(&mut self, evented: &mut dyn Source) -> ::std::io::Result<Token>;
}

impl Handle for PollWrapper {
    fn register(&mut self, evented: &mut dyn Source) -> ::std::io::Result<Token> {
        let token = Token(self.next_token_id);
        self.next_token_id += 1;
        self.poll
            .registry()
            .register(evented, token, Interest::READABLE | Interest::WRITABLE)?;
        self.tokens.push(token);
        Ok(token)
    }
}

impl<'a> Handle for PollHandle<'a> {
    fn register(&mut self, evented: &mut dyn Source) -> ::std::io::Result<Token> {
        let token = Token(*self.next_token_id);
        *self.next_token_id += 1;
        self.poll
            .registry()
            .register(evented, token, Interest::READABLE | Interest::WRITABLE)?;
        self.tokens.push(token);
        Ok(token)
    }
}
