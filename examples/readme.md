Use following cargo commands to run the server with a thread pool instead of async.

cargo run --example multi_threaded_server start -ip 127.0.0.1 -p 7878 -tp 10