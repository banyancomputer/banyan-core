# How to setup dev environment from scratch

## Clean Up / First Time Setup

Some cleanup from prior runs, assumes you're in the root of this repository:

```
rm -rf crates/banyan-core-service/data/s* \
  crates/banyan-core-service/data/uploads/* \
  crates/banyan-staging-service/data/server* \
  crates/banyan-staging-service/data/pl* \
  crates/banyan-staging-service/data/uploads/*
```

We need to generate the platform signing keys, this can be done by simply
starting up the core service and shutting it down (for running against the tomb CLI add "--features fake" flag):

```
cd crates/banyan-core-service
cargo run
```

The staging service has its own authentication material that needs to be
generated to authenticate back to the core service.

```
cd ../banyan-staging-service
cargo run
```

The services should now be ready to interact with each other. The steps so far
only need to be run once or when the environment needs to be reset. The
remaining instructions are for bringing up the project assuming everything is
setup as documented here.

## Front-End Setup

This is only needed if you want to bring up the web interface locally. From the
root of the repository.

```
cd crates/banyan-core-server/frontend
yarn install
cp .env.example .env.dev
```

Edit the `.env.dev` file and provide the `GOOGLE_CLIENT_ID=` and
`GOOGLE_CLIENT_SECRET=` values for the project. You'll need to get these from
someone that already has them. This will only work with the development keys,
using other keys won't allow authentication and shouldn't exist on developer's
systems.

You'll also need to ensure that your account has been granted access to the
OAuth2 project if we're still in the testing phase of our application. This
needs to be done by someone with access to the Google Cloud Console ([direct
deeplink to the relevant
page](https://console.cloud.google.com/apis/credentials/consent?authuser=3&project=core-services-a465d267)).

## Bringing Up Services

You'll want multiple terminals open (one for each of the two services, and
another if you'd like to run the front end). A terminal multiplexer like tmux
is recommended.

From the root of the repository:

To run the frontend switch to your third terminal and from the root of the
repository:

```
cd crates/banyan-core-service/frontend
yarn dev
```

You should now be able to open up your web-browser to
[http://127.0.0.1:3001](http://127.0.0.1:3001), login, and use the platform.

## Updating Tomb WASM

In the tomb repository go to the tomb-wasm sub-crate. Build it with the following command:

```
export BANYAN_CORE_CHECKOUT=~/workspace/banyan/banyan-core

wasm-pack build --release
rm -f ${BANYAN_CORE_CHECKOUT}/crates/banyan-core-service/frontend/tomb_build/
cp -f pkg/* ${BANYAN_CORE_CHECKOUT}/crates/banyan-core-service/frontend/tomb_build/
```

