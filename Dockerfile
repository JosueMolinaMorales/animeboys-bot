# Stage 1: Build the Rust Discord bot for arm32v7
# FROM rust:1.67 as builder

# RUN rustup target add armv7-unknown-linux-musleabihf
# RUN apt-get update && apt-get -y install binutils-arm-linux-gnueabihf
# RUN apt-get -y install musl-tools
# RUN apt-get -y install build-essential gcc-arm-linux-gnueabihf

# # Create a new empty shell project
# RUN USER=root cargo new --bin animeboys-bot
# WORKDIR /animeboys-bot

# # Copy over Cargo Files
# COPY ./.cargo ./.cargo
# COPY ./Cargo.lock ./Cargo.lock
# COPY ./Cargo.toml ./Cargo.toml

# # copy your source tree
# COPY ./src ./src

# # build
# RUN REALGCC=arm-linux-gnueabihf-gcc-8 \
#     TARGET_CC=musl-gcc \
#     cargo build --release --target armv7-unknown-linux-musleabihf

# # Stage 2: Create a minimal runtime image
# FROM rust:slim-buster

# # Set the working directory
# WORKDIR /animeboys-bot

# # Copy the compiled binary from the builder stage
# COPY --from=builder /animeboys-bot/target/armv7-unknown-linux-musleabihf/release/animeboys-bot ./

# # Set up any additional dependencies if required (e.g., if your bot uses external libraries)

# # Start the Discord bot
# CMD ["./animeboys-bot"]

# FROM rust:latest as builder
# RUN rustup target add armv7-unknown-linux-musleabihf
# RUN apt-get update && apt-get -y install binutils-arm-linux-gnueabihf
# WORKDIR /app

# COPY .cargo ./.cargo
# COPY Cargo.toml Cargo.lock ./

# COPY src ./src

# RUN cargo build --release --target armv7-unknown-linux-musleabihf

# FROM arm32v7/debian:buster-slim
# WORKDIR /app
# COPY --from=builder /app/target/armv7-unknown-linux-musleabihf/release/animeboys-bot ./

# CMD ["./animeboys-bot"]

# Stage 1: Build the Rust Discord bot for arm32v7
FROM arm32v7/rust as builder

WORKDIR /app

# Copy over Cargo Files
COPY ./.cargo ./.cargo
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Copy over source files
COPY ./src ./src

# Build
RUN cargo build --release

# Stage 2: Create a minimal runtime image
FROM debian:bullseye-slim

# Install any additional runtime dependencies
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/animeboys-bot ./

# Start the Discord bot
CMD ["./animeboys-bot"]