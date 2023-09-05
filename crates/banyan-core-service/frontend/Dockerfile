FROM docker.io/library/node:18-alpine AS base


FROM base AS deps
RUN apk add --no-cache libc6-compat

RUN mkdir /app
COPY package.json yarn.lock /app/
COPY tomb_build /app/tomb_build

WORKDIR /app

RUN yarn install


FROM base AS builder
WORKDIR /app

COPY . .
COPY --from=deps /app/node_modules /app/node_modules

ENV NEXT_TELEMETRY_DISABLED 1

RUN yarn build


FROM base AS runner

ENV NEXT_TELEMETRY_DISABLED 1
ENV NODE_ENV production

RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

COPY --from=builder /app/public /app/public

RUN mkdir /app/.next
RUN chown nextjs:nodejs /app/.next

COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone /app
COPY --from=builder --chown=nextjs:nodejs /app/.next/static /app/.next/static

USER nextjs
WORKDIR /app

EXPOSE 3000

ENV PORT 3000
ENV HOSTNAME "0.0.0.0"


CMD ["node", "server.js"]
