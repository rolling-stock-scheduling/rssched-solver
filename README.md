# Deployment via Docker 
- builing the docker image (from the main directory):

```bash
docker build --tag eth_rolling_stock_scheduling .
```

- running the server:

```bash
docker run --publish 3000:3000 eth_rolling_stock_scheduling
```

# Server Usage
- send `POST http://localhost:3000/solve' with a JSON body containing the input. After solving the solution is returned as JSON.
- send `GET http://localhost:3000/health' to see if the server is running.
- The tools Insomnia or Postman can send this requests.

# Single Run
- choose the instance in internal/src/main.rs
- from the main directory, compile and run the programm with:

```bash
cargo run --bin=single_run --release
```

# Start Server (without Docker)
- for the default port of 3000:

```bash
cargo run --bin=server --release
```

- for customized port:
```bash
cargo run --bin=server --release -- 4000
```

# Developement
- install the rust compiler rustc and the rust package manager cargo via rustup: https://www.rust-lang.org/tools/install
