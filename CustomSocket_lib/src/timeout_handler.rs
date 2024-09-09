use std::future::Future;

pub trait TimeoutHandler {
    fn timeouts_handler(&mut self, timeouts: Vec<String>) -> impl Future<Output = ()> + Send + Sync;
}

struct DefaultTimeoutHandler {
    pub timeout_amount: i32,
}

impl TimeoutHandler for DefaultTimeoutHandler {
    fn timeouts_handler(&mut self, timeouts: Vec<String>) -> impl Future<Output = ()> + Send + Sync{
        async {
            for timeout in timeouts {
                println!("Timeout on: {}", timeout);
                self.timeout_amount += 1;
            }
        }
    }
}