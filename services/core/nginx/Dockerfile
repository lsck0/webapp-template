FROM nginx:1.29.0

RUN apt-get update && apt-get install -y \
    autoconf \
    automake \
    g++ \
    git \
    libmaxminddb-dev \
    libmaxminddb0 \
    libmodsecurity-dev \
    libmodsecurity3 \
    libpcre2-dev \
    libpcre3-dev \
    libtool \
    libxml2 \
    libxml2-dev \
    libyajl-dev \
    make \
    mmdb-bin \
    pkg-config \
    xz-utils \
    zlib1g-dev

RUN git clone --depth 1 -b v3/master https://github.com/owasp-modsecurity/ModSecurity.git --recursive \
    && cd ModSecurity && ./build.sh && ./configure && make -j 16 && make install

RUN git clone --depth 1 https://github.com/owasp-modsecurity/ModSecurity-nginx.git /opt/ModSecurity-nginx

RUN git clone https://github.com/leev/ngx_http_geoip2_module /opt/ngx_http_geoip2_module

RUN curl -O http://nginx.org/download/nginx-1.29.0.tar.gz && \
    tar zxvf nginx-1.29.0.tar.gz && \
    cd nginx-1.29.0 && \
    ./configure --with-compat --add-dynamic-module=/opt/ngx_http_geoip2_module --add-dynamic-module=/opt/ModSecurity-nginx && \
    make modules && \
    cp -v objs/*.so /etc/nginx/modules/

RUN mkdir -p /etc/nginx/ssl /etc/ssl/private

COPY ./GeoIP /usr/share/GeoIP
RUN cd /usr/share/GeoIP && \
    xz -dv GeoLite2-City.mmdb.xz && \
    xz -dv GeoLite2-Country.mmdb.xz && \
    xz -dv GeoLite2-ASN.mmdb.xz

COPY ./ssl/san.cnf /etc/ssl/private/san.cnf
COPY ./ssl/selfsigned.key /etc/nginx/ssl/selfsigned.key
COPY ./ssl/selfsigned.crt /etc/nginx/ssl/selfsigned.crt

COPY ./nginx.conf /etc/nginx/nginx.conf

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD ["curl", "--fail", "--silent", "http://localhost:6666/health"]
