FROM rust:1.59-alpine


RUN apk add binaryen jq libxcb-dev libxkbcommon-dev musl-dev bash openssl-dev


COPY . /vaja_2
WORKDIR /vaja_2

RUN cargo build -r

RUN bash ./setup_web.sh
RUN bash ./build_web.sh

CMD ["./start_server.sh"]