# Deployment via Docker

- install the docker engine: https://docs.docker.com/engine/install/

- building the docker image (from the main directory):
  
  ```bash
  docker build --tag eth_scheduling_image .
  ```

- loading the image and running the server for the first time (removes old container of the same name):
  
  ```bash
  docker run --rm --env RAYON_NUM_THREADS=16 --publish 3000:3000 --name eth_scheduling_server eth_scheduling_image
  ```

- the server can use 16 threads and answers on port 3000.

- if the environment variable `RAYON_NUM_THREADS` is not set, the server will use as many threads as possible.

- short version (with a random name for the container):
  
  ```bash
  docker run -e RAYON_NUM_THREADS=16 -p 3000:3000 eth_scheduling_image
  ```

- stopping the docker container:
  
  ```bash
  docker stop eth_scheduling_server
  ```

- starting it again with
  
  ```bash
  docker start --attach eth_scheduling_server
  ```

- remove the container:
  
  ```bash
  docker container rm eth_scheduling_server
  ```

# Server Usage

- send `POST http://localhost:3000/solve` with a JSON body containing the input. After solving the solution is returned as JSON.

- send `GET http://localhost:3000/health` to see if the server is running.

- `Insomnia` or `Postman` can send this requests with a GUI.

- or `curl`:
  
  ```bash
  curl -X POST -H "Content-Type: application/json" -d @path/to/input.json http://localhost:3000/solve
  ```

# Single Run

- choose the instance in internal/src/main.rs

- from the main directory, compile and run the program with:
  
  ```bash
  cargo run --bin=single_run --release
  ```

- limiting the number of thread:
  
  ```bash
  RAYON_NUM_THREADS=16 cargo run --bin=single_run --release
  ```

# Start Server (without Docker)

- for the default port of 3000:
  
  ```bash
  cargo run --bin=server --release
  ```

- limiting the number of thread:
  
  ```bash
  RAYON_NUM_THREADS=16 cargo run --bin=server --release
  ```

- for customized port:
  
  ```bash
  cargo run --bin=server --release -- 4000
  ```

# Development

- install the rust compiler rustc and the rust package manager cargo via rustup: https://www.rust-lang.org/tools/install
