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

2. Prepare Database
```bash
cd crates/banyan-core-service
./scripts/prepare_queries.sh
```

3. Run the Rust Server + Migration
```bash
cargo run
```

4. Run the Frontend + Auth Server
```bash
cd crates/banyan-core-service/frontend
yarn start
```