FROM busybox:uclibc AS curl
ADD https://github.com/moparisthebest/static-curl/releases/latest/download/curl-amd64 /curl
RUN chmod +x /curl

FROM grafana/pyroscope:weekly-f155-97b7f968c

COPY --from=curl /curl /usr/bin/curl

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD ["/usr/bin/curl", "--fail", "--silent", "http://localhost:4040"]
