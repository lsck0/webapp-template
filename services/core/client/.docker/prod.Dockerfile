FROM node:24-alpine AS build
WORKDIR /app
COPY package*.json ./
RUN npm install --legacy-peer-deps
COPY . .
RUN npm run build

FROM nginx:stable-alpine AS runtime
COPY --from=build /app/dist /usr/share/nginx/html
CMD ["nginx", "-g", "daemon off;"]

EXPOSE 80

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD wget -q --spider http://localhost/ || exit 1
