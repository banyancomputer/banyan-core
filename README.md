# How to setup dev environment
From this directory, run:

1. 
````
cd crates/banyan-core-service;
cargo run --features fake;
# Wait for ^^ this to finish intializing the server and then exit it with Ctrl-C
```

2.
````
cp crates/banyan-core-service/data/signing-key.public crates/banyan-staging-service/data/platform-verifier.public;
````

3.
````
cd crates/banyan-staging-service;
cargo run -- generate-auth;
````

4.
````
cd crates/banyan-core-service;
./scripts/add_storage_host.sh;
````

4.
````
cd crates/banyan-core-service;
cargo run --features fake;
````

And in another terminal:

````
cd crates/banyan-staging-service;
cargo run;
````

5. 
If you need the frontend, run:
````
cd crates/banyan-core-service/frontend;
# Make sure you have a proper .env.dev file in this directory
source .env.dev;
yarn dev;
````
