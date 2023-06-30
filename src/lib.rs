use std::{
    fs,
    io::{prelude::*},
    net::{TcpStream, TcpListener},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
    error::Error,
    fmt,
};
//use std::any::type_name;

pub struct Server {
    listener: TcpListener,
    workers_pool: ThreadPool,
}

impl Server {
    /// Creates a new Server using an ip address and port
    /// 
    pub fn new(ip_address_port: &str, workers_pool_size: usize) -> Result<Self, Box<dyn Error>>  {
        let listener = TcpListener::bind(ip_address_port)?;
        let workers_pool = ThreadPool::new(workers_pool_size)?;
        Ok(Server {
            listener,
            workers_pool,
        })
    }

    /// Server starts serving clients
    /// 
    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        for stream in self.listener.incoming() {
            let stream = stream?;
            self.workers_pool.execute(|| {
                handle_connection(stream);
            });
        }
        Ok(())
    }
}

pub fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}


pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug)]
pub struct PoolCreationError;

impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Number of workers in pool must be greater than 0.")
    }
}

impl Error for PoolCreationError {}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size <= 0 {
            return Err(PoolCreationError);
        }

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Ok(ThreadPool {
            workers,
            sender: Some(sender),
        })
    
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn invalid_server_ip() {
        let invalid_ip_address = "127.0.1:7878";
        let result = Server::new(invalid_ip_address, 10);

        if let Err(e) = result {
            assert_eq!(e.to_string(), "No such host is known. (os error 11001)");
        } else {
            panic!("Expected Err, but got Ok");
        }

    }
    
}