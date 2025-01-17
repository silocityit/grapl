FROM rust:1-slim-buster AS base

ARG RUST_BUILD=debug

SHELL ["/bin/bash", "-c"]

# Necessary for rdkafka; pkg-config & libssl-dev for cargo-tarpaulin build
#
# Ignore this lint about deleteing the apt-get lists (we're caching!)
# hadolint ignore=DL3009
RUN --mount=type=cache,target=/var/lib/apt/lists \
    apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential=12.6 \
        libssl-dev=1.1.1d-0+deb10u7 \
        pkg-config=0.29-6 \
        zlib1g-dev=1:1.2.11.dfsg-1

# Install rust toolchain before copying sources to avoid unecessarily
# resinstalling on source file changes.
WORKDIR /grapl
COPY rust/rust-toolchain.toml rust/rust-toolchain.toml
WORKDIR /grapl/rust
# 'rustup show' will install components in the rust-toolchain.toml file
RUN rustup show

# copy sources
WORKDIR /grapl
COPY proto proto
COPY rust rust

WORKDIR /grapl/rust


# build
################################################################################
FROM base AS build

# Hadolint appears to be confused about some of these mount targets
# hadolint ignore=SC1091
RUN --mount=type=cache,target=/grapl/rust/target,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=secret,id=rust_env,dst=/grapl/env \
    if [ -f /grapl/env ]; then source /grapl/env; fi && \
    case "${RUST_BUILD}" in \
      debug) \
        cargo build ;; \
      release) \
        cargo build --release ;; \
      *) \
        echo "ERROR:  Unknown RUST_BUILD option: ${RUST_BUILD}"; \
        exit 1 ;; \
    esac

# Copy the build outputs to location that's not a cache mount.
# TODO: switch to using --out-dir when stable: https://github.com/rust-lang/cargo/issues/6790
RUN --mount=type=cache,target=/grapl/rust/target,sharing=locked \
    mkdir -p /outputs && \
    find "/grapl/rust/target/${RUST_BUILD}" -maxdepth 1 -type f -executable -exec cp {} /outputs \;


# build-test
# This target is not merged with the `build` target because the actions to run
# after cargo are different when building for tests and building the services, 
# and we'd rather not save all of the Rust `target/` directory to Docker image
# if we don't have to.
################################################################################
FROM base AS build-test

ENV RUST_INTEGRATION_TEST_FEATURES="node-identifier/integration,sqs-executor/integration,kafka-metrics-exporter/integration,model-plugin-deployer/integration,plugin-registry/integration"

# For test coverage reports
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install cargo-tarpaulin

# At the end of the test run, save the mount cache to the Docker image, as
# these files are needed for running the integration tests in containers.
#
# Hadolint appears to be confused about some of these mount targets
# hadolint ignore=SC1091
RUN --mount=type=cache,target=/grapl/rust/target,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=secret,id=rust_env,dst=/grapl/env \
    if [ -f /grapl/env ]; then source /grapl/env; fi && \
    case "${RUST_BUILD}" in \
      test) \
        cargo test --no-run ;; \
      test-integration) \
        cargo test --features "${RUST_INTEGRATION_TEST_FEATURES}" --test "*" --no-run ;; \
      *) \
        echo "ERROR: Unknown RUST_BUILD option: ${RUST_BUILD}"; \
        exit 1 ;; \
    esac && \
    cp -a /grapl/rust/target /tmp/target && \
    cp -a /usr/local/cargo/registry /tmp/registry

# Move the files back to where they belong.
# The mv command is apparently very slow here, cp and rm is much faster
# Ignore brace expansion warning; we're using bash as our shell, not sh
# hadolint ignore=SC3009
RUN cp -ar /tmp/target/. /grapl/rust/target/ && \
    cp -ar /tmp/registry/. /usr/local/cargo/registry/ && \
    rm -rf /tmp/{target,registry}

CMD [ "bash", "-c", "cargo test --features \"${RUST_INTEGRATION_TEST_FEATURES}\" --test \"*\" -- --nocapture" ]

# images for running services
################################################################################
# More information about the base image used here can be found at: 
# https://github.com/GoogleContainerTools/distroless/blob/main/cc/README.md.
# For debugging see: https://github.com/GoogleContainerTools/distroless#debug-images

# NOTE: we're using the debug containers at the moment so we have a
# shell; this lets us inject our Pulumi outputs in Local Grapl. If
# not for that, we could use the standard non-debug images.
FROM gcr.io/distroless/cc:debug AS rust-dist

USER nonroot

# analyzer-dispatcher
FROM rust-dist AS analyzer-dispatcher-deploy

COPY --from=build /outputs/analyzer-dispatcher /
ENTRYPOINT ["/analyzer-dispatcher"]

# generic-subgraph-generator
FROM rust-dist AS generic-subgraph-generator-deploy

COPY --from=build /outputs/generic-subgraph-generator /
ENTRYPOINT ["/generic-subgraph-generator"]

# graph-merger
FROM rust-dist AS graph-merger-deploy

COPY --from=build /outputs/graph-merger /
ENTRYPOINT ["/graph-merger"]

# plugin-work-queue
FROM rust-dist AS plugin-work-queue-deploy

COPY --from=build /outputs/plugin-work-queue /
ENTRYPOINT ["/plugin-work-queue"]


# plugin-registry
FROM rust-dist AS plugin-registry-deploy

COPY --from=build /outputs/plugin-registry /
ENTRYPOINT ["/plugin-registry"]

# node-identifier
FROM rust-dist AS node-identifier-deploy

COPY --from=build /outputs/node-identifier /
ENTRYPOINT ["/node-identifier"]

# node-identifier-retry
FROM rust-dist AS node-identifier-retry-deploy

COPY --from=build /outputs/node-identifier-retry /
ENTRYPOINT ["/node-identifier-retry"]

# sysmon-generator
FROM rust-dist AS sysmon-generator-deploy

COPY --from=build /outputs/sysmon-generator /
ENTRYPOINT ["/sysmon-generator"]

# osquery-generator
FROM rust-dist AS osquery-generator-deploy

COPY --from=build /outputs/osquery-generator /
ENTRYPOINT ["/osquery-generator"]

# web-ui
FROM rust-dist AS grapl-web-ui

COPY --from=build /outputs/grapl-web-ui /
COPY rust/grapl-web-ui/frontend /frontend
ENTRYPOINT ["/grapl-web-ui"]

# model plugin deployer
FROM rust-dist AS model-plugin-deployer
COPY --from=build /outputs/model-plugin-deployer /
ENTRYPOINT ["/model-plugin-deployer"]
