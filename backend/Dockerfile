FROM alpine:3.21.1

RUN apk add cargo rust
RUN apk add openssl-dev pkgconfig

WORKDIR /app

COPY . .

RUN cargo build -r

CMD cargo run -r