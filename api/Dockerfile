FROM --platform=$BUILDPLATFORM ghcr.io/drogue-iot/builder:0.1.20 as builder

RUN mkdir /build
ADD . /build
WORKDIR /build

RUN cargo build --release

FROM registry.access.redhat.com/ubi8-minimal

LABEL org.opencontainers.image.source="https://github.com/lulf/doorlock"

COPY --from=builder /build/target/release/doorlock-api /
COPY --from=builder /build/run.sh /

ENTRYPOINT [ "/run.sh" ]
