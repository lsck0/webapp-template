FROM busybox:uclibc AS curl
ADD https://github.com/moparisthebest/static-curl/releases/latest/download/curl-amd64 /curl
RUN chmod +x /curl

FROM nginx/nginx-prometheus-exporter:latest

COPY --from=curl /curl /usr/bin/curl

CMD ["-nginx.scrape-uri=http://nginx:6666/status"]

EXPOSE 9113

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD ["/usr/bin/curl", "--fail", "--silent", "http://localhost:9113/metrics"]
