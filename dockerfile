FROM paritytech/ci-linux:974ba3ac-20201006 as builder
LABEL description="This is the build stage for Polkadot. Here we create the binary."

ARG PROFILE=release
WORKDIR /hmt

COPY . /hmt

RUN cargo build --$PROFILE

# ===== SECOND STAGE ======

FROM debian:buster-slim
LABEL description="This is the 2nd stage: a very small image where we copy the Polkadot binary."
ARG PROFILE=release
COPY --from=builder /hmt/target/$PROFILE/node-template /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /hmt hmt && \
	mkdir -p /hmt/.local/share/hmt && \
	chown -R hmt:hmt /hmt/.local && \
	ln -s /hmt/.local/share/hmt /data && \
	rm -rf /usr/bin /usr/sbin

USER hmt
EXPOSE 30333 9933 9944
VOLUME ["/data"]

CMD ["/usr/local/bin/hmt"]