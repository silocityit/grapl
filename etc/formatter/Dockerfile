FROM debian:buster-slim AS formatter

# Ignore this lint about deleteing the apt-get lists (we're caching!)
# hadolint ignore=DL3009
RUN --mount=type=cache,target=/var/lib/apt/lists \
    apt-get update \
    && apt-get install --yes --no-install-recommends \
        ca-certificates=20200601~deb10u2 \
        curl=7.64.0-4+deb10u2 \
        netbase=5.6

# Because of the pipe below
SHELL ["/bin/bash", "-c", "-o", "pipefail"]

# Install npm, Prettier
RUN curl -fsSL https://deb.nodesource.com/setup_16.x | bash -

# Ignore DL3009 lint about deleteing the apt-get lists (we're caching!)
# Ignore DL3008 lint about pinning NodeJS; it's complex: https://github.com/nodesource/distributions/issues/33
# hadolint ignore=DL3009,DL3008
RUN --mount=type=cache,target=/var/lib/apt/lists \
    apt-get update && \
    apt-get install --yes --no-install-recommends nodejs \
    && npm install -g prettier@2.2.1 \
    && npm install -g prettier-plugin-toml@0.3.1

# TODO:
# In later iterations, we can also install all the Rustup cargofmt stuff here
# as well as Pants to perform containerized linting.
