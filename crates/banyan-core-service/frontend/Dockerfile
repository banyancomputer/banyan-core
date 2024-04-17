FROM docker.io/library/node:18-slim AS base
RUN apt-get update -y && apt-get install sqlite3 -y

ARG API_URL
ARG DATA_URL

RUN mkdir /app

COPY package.json package-lock.lock /app/
COPY tomb_build /app/tomb_build

ENV NEXT_TELEMETRY_DISABLED 1

WORKDIR /app

RUN npm ci

COPY . .

ENV NEXT_PUBLIC_API_URL=$API_URL
ENV NEXT_PUBLIC_DATA_URL=$DATA_URL
ENV NODE_ENV production

RUN npm run build

RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

RUN chown nextjs:nodejs /app/.next

USER nextjs

EXPOSE 3000

ENV PORT 3000
ENV HOSTNAME "0.0.0.0"

CMD ["npm", "start"]
