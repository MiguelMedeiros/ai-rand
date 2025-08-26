# Use the official Rust image (latest stable)
FROM rust:latest

# Set working directory
WORKDIR /app

# Copy project files
COPY . .

# Build the application
RUN cargo build --release

# Copy the knowledge base to the working directory
COPY knowledge-base.txt ./

# Copy and load environment variables from .env file
COPY .env ./

# Run the bot
CMD ["./target/release/client-pubky"]
