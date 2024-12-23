when you run the test run it using the command "cargo test -- --test-threads=1"
as when you just run cargo test some test failed due to some unknown errors like os errors:
1- An existing connection was forcibly closed by the remote host. (os error 10054) 
2- An established connection was aborted by the software in your host machine. (os error 10053) 

note that this command only make the threads to run after each other but the concept of multithreading itself remains at it is in the code implemtati
