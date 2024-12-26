FROM node:24-alpine

WORKDIR /app

COPY package*.json ./

RUN npm install --legacy-peer-deps

COPY . .

CMD ["npm", "run", "dev"]

EXPOSE 80

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD wget -q --spider http://localhost/ || exit 1
