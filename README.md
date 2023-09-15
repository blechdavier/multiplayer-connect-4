# multiplayer-connect-4

A Rust implementation of the game Connect 4 using a client-server architecture.

# Usage

1. Clone the repository
2. Run different binaries using `bin --client` or `bin --server`
3. If running the server, make sure to forward port 60941
4. Follow command line prompts in the client to connect to the server

# Features

* Custom protocol
* Custom serializer and deserializer
* Automatic board evaluation and game scoring

# Note

Although the server is multithreaded and should in theory support running multiple games at once using `tokio::spawn`,
there is still unknown behavior when multiple games are happening at once, including moves being played on multiple
boards at once.
  
