use std::time::Duration;
use crossbeam_channel::after;

pub fn with_timeout<T, F>(timeout_secs: u64, f: F) -> Result<T, &'static str> 
where 
    F: FnOnce() -> T,
{
    let timeout = after(Duration::from_secs(timeout_secs));
    println!("Running test with timeout of {} seconds", timeout_secs);
    crossbeam_channel::select! {
        recv(timeout) -> _ => {
            println!("Test timed out after {} seconds", timeout_secs);
            Err("Test timed out after 15 seconds")
        }
        default => {
            Ok(f())
        }
    }
}