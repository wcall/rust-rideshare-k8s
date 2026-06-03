FROM rust:latest as deps

# Force frame pointers so eBPF profilers (Pyroscope/Alloy, Parca) can unwind
# stacks regardless of the optimization level used in [profile.release].
ENV RUSTFLAGS="-C force-frame-pointers=yes"

WORKDIR /usr/src/server
# Copy only files needed for dependency resolution
COPY server/Cargo.toml server/Cargo.lock ./
# Create a dummy main.rs to compile dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

FROM deps as builder
# Now copy the actual source code
COPY server/src ./src
# Build the application
RUN cargo install --profile release --path .

FROM rust:slim as runtime
COPY --from=builder /usr/local/cargo/bin/server /usr/local/bin/server
EXPOSE 5000
CMD ["server"]
