#!/bin/bash
set -euo pipefail
IFS=$'\n\t'

podman run -it --rm \
    --name jaeger \
    -e COLLECTOR_OTLP_ENABLED=true \
    -p 16686:16686 \
    -p 14269:14269 \
    -p 4317:4317 \
    docker://jaegertracing/all-in-one:1
