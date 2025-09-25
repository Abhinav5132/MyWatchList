use std::thread;
use crate::{backend::setup_backend, frontend::launch_frontent};
pub mod backend;
pub mod frontend;

pub fn main() {
    dotenvy::dotenv().ok();

    thread::spawn(|| {
        if let Err(err) = setup_backend(){
            dbg!("Backend error");
            dbg!(err);
        }
    });
    
    launch_frontent();

}