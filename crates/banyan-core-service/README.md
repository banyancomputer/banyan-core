# Setting up Dev Environment
## Prerequisites
- [Cargo](https://www.rust-lang.org/tools/install)
- [Yarn](https://classic.yarnpkg.com/en/docs/install/#debian-stable)
- [Sqlx CLI](https://docs.rs/crate/sqlx-cli/0.5.7)

## Setup
1. Install Frontend Dependencies
```bash
cd crates/banyan-core-service/frontend
yarn install
```

2. 
You'll also need to setup NextAuth credentials. See [here](frontend/README.md) for more information.

3. Prepare Database
```bash
cd crates/banyan-core-service
./scripts/prepare_queries.sh
```

4. Run the Rust Server + Migration
```bash
cargo run
```

5. Run the Frontend + Auth Server
```bash
cd crates/banyan-core-service/frontend
yarn dev
```

6. Allow your email to access the frontend -- make sure this email is allowed to sign in per the OAuth provider and client you're using.
```bash
curl -X POST \
-H "Content-Type: application/json" \
-d '{ "email": "<your_google_email>" }' \
 "http://localhost:3000/api/admin/allow"
```

7. Navigate to [http://localhost:3000](http://localhost:3000) in your browser. Login with google.

## Running Tests
### prepare test scripts
```bash
cd scripts/js
yarn install
```

Register a device to use for testing:

```bash
./scripts/register_test.sh
```

You should see a JSON response that contains an account id in your browser.
Make sure to export the account id as an environment variable:
```bash
export ACCOUNT_ID=<account_id>
```

Now you can run the auth test:
```bash
./scripts/auth_test.sh
```



