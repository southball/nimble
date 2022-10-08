FROM node:16-buster-slim

# Copy package.json and yarn.lock so it can be cached
COPY ./package.json ./package.json
COPY ./yarn.lock ./yarn.lock
COPY ./app/package.json ./app/package.json
COPY ./app/yarn.lock ./app/yarn.lock
RUN yarn && cd app && yarn

COPY . .
RUN cd app && yarn build && rm -rf node_modules

CMD ["yarn", "start"]
