FROM rust:latest as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock /app/
COPY src /app/src/
RUN cargo build --release

FROM debian:bullseye-slim
#RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
#FROM debian:latest 
WORKDIR /app
RUN mkdir -p /app/tilecache
RUN mkdir -p /app/projects/vedur
COPY projects/vedur/WMTSCapabilities.xml /app/projects/vedur/
COPY --from=builder /app/target/release/stackmap-wmts /app/stackmap-wmts
CMD /app/stackmap-wmts


