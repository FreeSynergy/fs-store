FROM fedora:41

WORKDIR /app

# Copy pre-built binaries (built by CI)
COPY crates/fs-store-cli/target/release/fs-store-cli /usr/local/bin/fs-store-cli

# Runtime user
RUN useradd -r -s /sbin/nologin fsstore

USER fsstore

EXPOSE 8082 9092

ENTRYPOINT ["/usr/local/bin/fs-store-cli"]
CMD ["serve"]

LABEL org.opencontainers.image.source="https://github.com/FreeSynergy/fs-store"
LABEL org.opencontainers.image.description="FreeSynergy Store"
