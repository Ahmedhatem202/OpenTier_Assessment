1- By runing the command "cargo test" on the powershell this is my output:

    PS C:\Users\user\Documents\New folder\embedded-recruitment-task-0.0.1> cargo test
   Compiling embedded-recruitment-task v0.1.0 (C:\Users\user\Documents\New folder\embedded-recruitment-task-0.0.1)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.20s
     Running unittests src\lib.rs (target\debug\deps\embedded_recruitment_task-dfcd91e22dfb2448.exe)

    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

        Running tests\client.rs (target\debug\deps\client-ca188493148c5487.exe)

    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

        Running tests\client_test.rs (target\debug\deps\client_test-02364f82aa9a60f2.exe)

    running 5 tests
    test test_client_add_request ... ok
    test test_client_connection ... ok
    test test_client_echo_message ... ok
    test test_multiple_clients ... ok
    test test_multiple_echo_messages ... ok

    test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.23s

    Doc-tests embedded_recruitment_task

So, every test is executed as expected.

2- first i made sure that only one client access the socket at a time and also the server send or read a message for one thread only at a time, all this using a mutex variable 

3- second i made a change in the function run so that for each connection a new thread is created to handle each client individually.



