# Build stagw
FROM rust:1.83 AS builder

WORKDIR /usr/src/photolabeler

COPY . .

RUN cargo build --release

# Start a new stage to create a smaller image without unnecessary build dependencies
FROM debian:bookworm-slim

# Set the working directory
WORKDIR /usr/src/photolabeler

# Copy the built binary from the previous stage
COPY --from=builder /usr/src/photolabeler/target/release/photolabeler ./
COPY --from=builder /usr/src/photolabeler/templates ./templates

# Command to run the application
CMD ["./photolabeler"]

EXPOSE 3071
