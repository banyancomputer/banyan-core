FROM docker.io/library/node:18-slim AS base
RUN apt-get update -y && apt-get install sqlite3 -y

ARG API_URL
ARG DATA_URL

RUN mkdir /app

COPY package.json yarn.lock /app/
COPY tomb_build /app/tomb_build

ENV NEXT_TELEMETRY_DISABLED 1

WORKDIR /app

RUN yarn install

COPY . .

ENV NEXT_PUBLIC_API_URL=$API_URL
ENV NEXT_PUBLIC_DATA_URL=$DATA_URL
ENV NODE_ENV production

RUN yarn build

RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

RUN chown nextjs:nodejs /app/.next

USER nextjs

EXPOSE 3000

ENV PORT 3000
ENV HOSTNAME "0.0.0.0"

CMD ["yarn", "start"]
