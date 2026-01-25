use tokio::sync::Notify;

pub use crate::{backend::setup_backend, frontend::launch_frontend};
use std::sync::{Arc, Mutex, mpsc};
pub use std::thread;
pub mod backend;
pub mod frontend;

pub fn main() {
    dotenvy::dotenv().ok();

    let (tx , rx) = mpsc::channel();
    thread::spawn(move ||{
        if let Err(err) = setup_backend(tx) {
            dbg!("Backend error");
            dbg!(err);
        }
    });
    
    let _ = rx.recv();
    
    launch_frontend();
    
}