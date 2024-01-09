# Use the official Rust image as the base image
FROM rust:latest AS builder

# Set the working directory inside the container
WORKDIR /app

# Copy the code into the container
COPY internal/ internal/
COPY model/ model/
COPY objective_framework/ objective_framework/
COPY server/ server/
COPY solution/ solution/
COPY solver/ solver/
COPY time/ time/
COPY Cargo.toml Cargo.toml

# Compile the Rust code
RUN cargo build --bin=server --release

# Use the official Debian slim image as the base runtime image
FROM debian:stable-slim AS runtime

# Set the working directory inside the container
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/server .

# Expose the port specified in the CMD (or default to 3000)
EXPOSE 3000

# Run the compiled binary when the container starts
ENTRYPOINT ["./server"]
