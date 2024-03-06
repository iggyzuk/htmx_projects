# Use a Rust base image for building
FROM rust:latest AS builder

# Set the working directory inside the container
WORKDIR /usr/src/myapp

# Copy the source code into the container
COPY . .

# Build the Rust application
RUN cargo build --bin htmx_wordle --release

# Create a new lightweight image for running the application
FROM debian:stable-slim AS runtime

# Set the working directory inside the container
WORKDIR /usr/src/myapp/

# Copy the compiled binary from the builder stage into the final image
COPY --from=builder /usr/src/myapp/target/release/htmx_wordle ./app

# Expose any necessary ports
EXPOSE 3242

# Command to run the application
CMD ["./app"]
