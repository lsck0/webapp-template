FROM node:24-alpine

WORKDIR /app

COPY package*.json ./

RUN npm install --legacy-peer-deps

COPY . .

RUN npx -y tsc

RUN npx -y eslint .

RUN npm run test

CMD ["npm", "run", "dev"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=30s \
    CMD wget --spider http://localhost:80/ || exit 1
