FROM rust:1.59-alpine AS builder
RUN apk add binaryen jq libxcb-dev libxkbcommon-dev musl-dev bash openssl-dev
COPY . /vaja_2
WORKDIR /vaja_2
RUN cargo build -r
RUN bash ./setup_web.sh
RUN bash ./build_web.sh


FROM node:16-alpine
COPY --from=builder /vaja_2/docs ./docs
COPY --from=builder /vaja_2/run.js ./
CMD ["node", "./run"]  



