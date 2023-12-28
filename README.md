# 🌴 Banyan Core

This repository contains Banyan Computer's internal platform services.

## 💻 How to setup dev environment from scratch

This document will help you set up a new development environment to contribute
to this project. Please open a pull request if these instructions are
out-of-date.

### 📦 Dependencies

You should have a working Rust toolchain, yarn, docker engine (or equivalent), and sqlite installed on your
machine.

### 🐳 Docker (or equivalent)

You must have an up-to-date version of docker installed on your machine to fully run the development environment.

Make sure the following command work:

```sh
docker --version
```

If you need to install docker, please refer to the [official documentation](https://docs.docker.com/engine/install/)

### 💰 Install `sqlx-cli`

This project relies upon [`sqlx`](https://crates.io/crates/sqlx) to interact
with SQL datases from Rust code. The sqlx command-line interface can be
installed using the following command.

```sh
cargo install sqlx-cli --no-default-features --features completions,native-tls,sqlite
```

*NB:* you may replace the `native-tls` flag with the `openssl-vendored` flag if
you would like to use a vendored copy of the `openssl` library instead of your
system's version. Refer to the crate's manifest
[here](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/Cargo.toml) to
see additional features.

### 🔧 Environment Setup

#### Core Service 👑

The core service uses a `.env` file to store environment variables. A sample
file exists as a starting point. You can copy that sample file into place
by running the following command:

```sh
cp crates/banyan-core-service/.env{.sample,}
```

Next, edit the `.env` file and provide the `GOOGLE_CLIENT_ID=` and
`GOOGLE_CLIENT_SECRET=` values for the project. You will need to get these from
someone that already has them.

This will only work with the development keys,
using other keys won't allow authentication and shouldn't exist on developer's
systems.

You'll also need to ensure that your account has been granted access to the
OAuth2 project if we're still in the testing phase of our application. This
needs to be done by someone with access to the Google Cloud Console ([direct
deeplink to the relevant
page](https://console.cloud.google.com/apis/credentials/consent?authuser=3&project=core-services-a465d267)).

#### Staging Service and Storage Provider Service 📦

The staging service and storage provider service use a `.env` file to store 
an argument called `UPLOAD_STORE_URL`. This argument is used to connect to 
an Object Store backend. This variable must be present at runtime, or the service 
will fail to start. This variable specifies a connection to the Object Store
backend the given service should use at runtime.

A sample file exists in either crate. This file should configure a valid connection
to a local filesystem Object Store at the path `./data/uploads` within either crate. You can copy those 
sample files into place by running the following command: 

```sh
cp crates/banyan-staging-service/.env{.sample,}
cp crates/banyan-storage-provider-service/.env{.sample,}
```
You must copy these defaults in place before attempting to run either service or resetting your
devlopment environment with `./bin/reset_env.sh` (discussed below).

These services also support using S3 as an Object Store backend. For 
development purposes this project initializes a MinIo backend that is spawned
locally by `./bin/reset_env.sh`.

The `.env.sample` files in either crate should contain example connection strings
for connecting to this local MinIo instance in your development environment. 
DO NOT USE THE SPECIFIED CREDENTIALS IN PRODUCTION. If you've run`./bin/reset_env.sh`
successfully then MinIo should be available at those example endpoints.

You can check the status of the MinIo container by running:

```
docker ps -a
```

You should see a container named `banyan-minio` running. If not, make sure docker
is installed and properly configured in your environment.

### ✨ Automatic Clean Up / First Time Setup

The easiest way to set up a development environment is to use the
`reset_env.sh` helper script, which automatically performs the steps outlined
in this document below.

```sh
./bin/reset_env.sh
```

### 🐣 Manual Clean Up / First Time Setup

Some cleanup from prior runs, assumes you're in the root of this repository:

```sh
make clean
```

We need to generate the platform signing keys, this can be done by simply
starting up the core service and shutting it down. The
`generate-core-service-key` command will do this for you. To run against the
tomb CLI add the `--features fake` flag.

After that command has finished, the platform's public key should be copied
over to the `data/` directories of the staging and storage provider services.

```sh
make generate-core-service-key
cp -f crates/banyan-core-service/data/service-key.public crates/banyan-staging-service/data/platform-key.public
cp -f crates/banyan-core-service/data/service-key.public crates/banyan-storage-provider-service/data/platform-key.public
```

The staging and storage provider services have their own authentication
material that must be generated to authenticate with the core service. Once
again, this can be done by starting up the services and then shutting them
down. The `generate-staging-service-key` and
`generate-storage-provider-service-key` commands will perform this step for
you.

```sh
make generate-staging-service-key
make generate-storage-provider-service-key
```

Now, the staging and storage provider services must be registered in the sqlite
database as potential storage providers. This can be done by running the
following scipts:

```
source bin/add_staging_host.sh
source bin/add_storage_host.sh
```

The services should now be ready to interact with each other. The steps so far
only need to be run once or when the environment needs to be reset. The
remaining instructions are for bringing up the project assuming everything is
setup as documented here.

#### 🪟 Front-End Setup

Run `yarn install` to install JavaScript dependencies for the frontend.

```sh
cd crates/banyan-core-service/frontend
yarn install
cd -
```

This is only needed if you want to bring up the web interface locally.

**NB:** This command should be run once more whenever changes have been made to
the frontend.

## 🚀 Bringing Up Services

**📍 TIP:** a terminal multiplexer like tmux is recommended for this step.

To build the web interface, run these commands from the root of the repository:

```sh
cd crates/banyan-core-service/frontend
yarn build
cd -
```

In one terminal, start the core service by running:

```sh
cd crates/banyan-core-service
cargo run
```

In another terminal, start the staging service by running:

```sh
cd ../banyan-staging-service
cargo run
```

In another terminal, start the storage provider service by running:

```sh
cd crates/banyan-storage-provider-service
cargo run
```

You should now be able to open up your web-browser to
[http://127.0.0.1:3001](http://127.0.0.1:3001), login, and use the platform.

## 🔌 Connecting to the Databases

It can be helpful during development to query the SQL databases for the
core, staging, and storage provider services directly.

Each of these services' databases write to their respective `data/` directory.
The `Makefile` contains some commands to attach a sqlite prompt to each:

```sh
make connect-to-core-database
make connect-to-staging-database
make connect-to-storage-provider-database
```

### 💭 Helpful Commands

Use these commands to list databases, indexes, and tables.

* `.databases`: list names and files of attached databases
* `.indexes`: list names of indexes
* `.tables`: list names of tables
* `.schema`: show the `CREATE` statements for table(s)

For more information run `.help` in the sqlite prompt, or refer to the
[Official Documentation](https://www.sqlite.org/docs.html).

### 🐚 Refreshing the `.sqlx` Cache

The `sqlx` library places some files within a `.sqlx/` directory in order to
typecheck our queries. Sometimes, this cache may need to be refreshed.

Run the `bin/prepare_queries.sh` script(s) if you encounter this error.

## 🔒 Updating Tomb WASM

In the tomb repository go to the `tomb-wasm` sub-crate. Build it with the
following command:

```sh
export BANYAN_CORE_CHECKOUT=~/workspace/banyan/banyan-core
wasm-pack build --release
rm -f ${BANYAN_CORE_CHECKOUT}/crates/banyan-core-service/frontend/tomb_build/
cp -f pkg/* ${BANYAN_CORE_CHECKOUT}/crates/banyan-core-service/frontend/tomb_build/
```


## Database Schema

### Core Service
![Core_Service Diagram](./docs/images/db-core-service.png)

### Staging Service
![Staging Service Diagram](./docs/images/db-staging-service.png)
