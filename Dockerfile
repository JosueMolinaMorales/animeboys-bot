# Stage 1: Build the Rust Discord bot for arm32v7
FROM arm32v7/rust:latest as builder

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files to leverage Docker layer caching
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY ./src ./src

# Build the Rust application
RUN cargo build --release

# Stage 2: Create a minimal runtime image
FROM arm32v7/rust:slim-buster

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/animeboys-bot .

# Set up any additional dependencies if required (e.g., if your bot uses external libraries)

# Start the Discord bot (replace with the actual command to start your bot)
CMD ["./animeboys-bot"]