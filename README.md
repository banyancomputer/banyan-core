# How to setup dev environment from scratch

## Clean Up / First Time Setup

Some cleanup from prior runs, assumes you're in the root of this repository:

```
rm -rf crates/banyan-core-service/data/s* \
  crates/banyan-core-service/data/uploads/* \
  crates/banyan-staging-service/data/pl* \
  crates/banyan-staging-service/data/uploads/*
```

We need to generate the platform signing keys, this can be done by simply
starting up the core service and shutting it down:

````
cd crates/banyan-core-service
cargo run
```

Once you see the 'server listening' line, press Ctrl-C to exit the application.
We need to provide the public portion of this to the staging service so it can
validate storage grants issued by our platform:

````
cp ./data/signing-key.public ../banyan-staging-service/data/platform-verifier.public
````

The staging service has its own authentication material that needs to be
generated to authenticate back to the core service.

````
cd ../banyan-staging-service
cargo run -- --generate-auth
````

This will exit automatically after generating the required keys so nothing else
needs to be done here. We need to go back to the core service and inform it of
the staging service's credentials. There is a script that will load this up
automatically. This script assumes the staging service will be accessible to
clients at the address http://127.0.0.1:3002 which we'll need later when
starting the service up.

````
cd ../banyan-core-service
./scripts/add_storage_host.sh;
````

The services should now be ready to interact with each other. The steps so far
only need to be run once or when the environment needs to be reset. The
remaining instructions are for bringing up the project assuming everything is
setup as documented here.

## Front-End Setup

This is only needed if you want to bring up the web interface locally. From the
root of the repository. You'll also want to take care to replace `$YOUR_EMAIL`
with the Google email you're going to try to use to login:

```
cd crates/banyan-core-server/frontend
yarn install
./scripts/allow_list.sh $YOUR_EMAIL
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

````
cd crates/banyan-core-service
cargo run --features fake
````

This will start up the core service running locally on port 3001, and all the
use of a fake authentication provider that does not depend on Google auth. The
fake authentication provider is used by tests, it doesn't disable the normal
authentication mechanism if you'd prefer to use that.

In the next terminal bring up the staging service, from the root of the
repository. The address and port came from the assumption earlier that this is
where the imported key's service lives.

````
cd crates/banyan-staging-service
cargo run -- --listen 127.0.0.1:3002 --db-url sqlite://./data/server.db
````

Here we're listening on the address the script registered for the staging
service earlier. These must match for our services to interoperate.

To run the frontend switch to your third terminal and from the root of the
repository:

```
cd crates/banyan-core-service/frontend
source .env.dev
yard run dev
```

You should now be able to open up your web-browser to
[http://127.0.0.1:3000](http://127.0.0.1:3000), login, and use the platform.

Note: It seems like the front end is not automatically creating and escrowing
keys right now... So I couldn't get it working.
