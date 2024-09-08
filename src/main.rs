use std::future::Future;
use std::marker::PhantomData;
use std::os::unix::raw::off_t;
use std::path::Iter;
use std::sync::Arc;
use std::thread::spawn;
use std::time::Duration;
use tokio::io::{self, AsyncBufReadExt};
use CustomServer::CustomServer;
use tokio::sync::{Mutex, Notify};
use tokio::task::JoinHandle;
use CustomSocket_lib::*;
use CustomSocket_lib::packet::Packet;
use futures::future::join_all;

pub struct DefaultTimeoutHandler {
    pub timeout_amount: i32,
}

impl TimeoutHandler for DefaultTimeoutHandler {
    fn timeouts_handler(&mut self, timeouts: Vec<String>) -> impl Future<Output = ()> + Send + Sync {
        async {
            for timeout in timeouts {
                println!("Timeout on: {}", timeout);
                self.timeout_amount += 1;
            }
        }
    }
}

struct MyTimeoutHandler {
    socket_send: Option<Arc<CustomSocket>>,
}

impl MyTimeoutHandler {
    fn set_socket(&mut self, socket: Arc<CustomSocket>) {
        self.socket_send = Some(socket);
    }

    fn new() -> Self {
        MyTimeoutHandler {
            socket_send : None,
        }
    }
}

impl TimeoutHandler for MyTimeoutHandler {
    fn timeouts_handler(&mut self, timeouts: Vec<String>) -> impl Future<Output=()> + Send + Sync {
        async {
            for timeout in timeouts {
                println!("timeout {}", timeout);
                let a_m: Vec<&str> = timeout.split("|").collect();
                let i_p: Vec<&str> = a_m[0].clone().split(":").collect();
                let (ip, port) = (i_p[0].clone().to_string(), i_p[1].clone().parse::<u16>().unwrap());
                println!("Try to send on {}:{}", ip, port);
                match &self.socket_send {
                    None => {
                        panic!("There is no socket in timeouthandler")
                    }
                    Some(socket) => {
                        socket.send(ip.to_string(), port, "Timeout".as_bytes().to_vec(), 100).await.unwrap()
                    }
                }
            }
        }
    }
}
impl DefaultTimeoutHandler {
    fn new() -> Self {
        DefaultTimeoutHandler {
            timeout_amount: 0,
        }
    }
}

#[tokio::main]
async fn main() {
    /*
    let mut futures: Vec<_> = vec![];
    for i in 0..100 {
        let future = async move {
            let shared_ready = Arc::new(Notify::new());
            let mut socket = CustomSocket::new("127.0.0.1".to_string(), 8091 + i, SocketType::Send, shared_ready.clone(), Arc::new(Mutex::new(None)));
            socket.connect().await.unwrap();

            println!("Sned");
            socket.send("127.0.0.1".to_string(), 8090, (1..50).collect(), 0).await.unwrap()
        };

        futures.push(future);
    }

    join_all(futures).await;

    let shared_ready = Arc::new(Notify::new());
    let mut socket = CustomSocket::new("127.0.0.1".to_string(), 8091, SocketType::Send, shared_ready.clone(), Arc::new(Mutex::new(None)));
    socket.connect().await.unwrap();

    println!("Sned");
    socket.send("127.0.0.1".to_string(), 8090, (1..50).collect(), 13).await.unwrap();
    println!("WOOOW");
     */
    let timeout_handler = MyTimeoutHandler::new();
    let mut server = Arc::new(CustomServer::new("127.0.0.1".to_string(), 8090, "127.0.0.1".to_string(), 8091, timeout_handler).await);

    server.timeout_handler.lock().await.set_socket(server.get_ss());

    let _server = server.clone();
    let recv = _server.start();


    let send = tokio::spawn(async move {
        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin).lines();

        while let Ok(Some(line)) = reader.next_line().await {
            println!("{}", line);
            server.send("127.0.0.1".to_string(), 8090, line.as_bytes().to_vec()).await;
        }
    });

    tokio::join!(recv, send);

}