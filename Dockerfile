# Use the official Rust image as the base image
FROM rust:1.76 as builder

# Create a new binary project
RUN USER=root cargo new --bin actix_web_app
WORKDIR /actix_web_app

# Copy the Cargo.toml and Cargo.lock files into the container
COPY ./Cargo.toml ./Cargo.lock ./

# Cache the dependencies - this step ensures that your dependencies
# are cached unless changes to one of the two Cargo files are made.
RUN cargo fetch --locked

# Copy the source code of your application into the container
COPY ./src ./src

# Build your application in release mode
RUN cargo build --release

# Use Debian buster-slim as the runtime base image
FROM debian:buster-slim

# Install OpenSSL - required by Actix Web
RUN apt-get update \
    && apt-get install -y openssl gcc ca-certificates build-essential libffi-dev bc sysstat\
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage to the runtime stage
COPY --from=builder /actix_web_app/target/release/actix_web_app /usr/local/bin/

# Expose the port on which your server will run
EXPOSE 18000

# Command to run the binary
CMD ["actix_web_app"]
