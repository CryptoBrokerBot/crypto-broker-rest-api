FROM debian:latest as builder
ADD . .
RUN apt update
RUN apt-get install -y build-essential curl
RUN apt update
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder ./target/release/my-program ./cb-rest
EXPOSE 8080
CMD ["./cb-rest"]

