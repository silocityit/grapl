# Every time we bring up Localstack, it goes and downloads Elasticmq.
# This "fork" just preloads it in the right spot.
# Saves us time on bringing up Localstack,
# and avoids an issue wimax@ ran into when running under Nomad.

FROM localstack/localstack-full:0.12.15 AS localstack-base
RUN mkdir -p /opt/code/localstack/localstack/infra/elasticmq/ \
    && wget --quiet https://s3-eu-west-1.amazonaws.com/softwaremill-public/elasticmq-server-1.1.0.jar \
    && mv elasticmq-server-1.1.0.jar /opt/code/localstack/localstack/infra/elasticmq/elasticmq-server.jar \
