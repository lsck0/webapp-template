FROM docker.io/ollama/ollama:0.15.4
# FROM docker.io/ollama/ollama:0.15.4-rocm

COPY ./start.sh /start.sh

ENTRYPOINT ["/start.sh"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD ollama list || exit 1
