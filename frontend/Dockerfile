FROM alpine:3.21.1

WORKDIR /app

RUN apk add npm nodejs

COPY package.json package-lock.json /app/

RUN npm ci

CMD npm run dev -- --host