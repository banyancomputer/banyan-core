# Tomb-WWW

Experimental frontend for Tomb.

## Dependencies

- [Node.js](https://nodejs.org/en/)
- [Yarn](https://yarnpkg.com/)
- Next.js
- Docker

## Development Setup

Install dependencies:

```bash
yarn install
```

### Environment and Services

There is a `.env.example` file in the root of the project. Copy this file to `.env.dev` and fill in the values. See below for more information on each variable.

#### **Next Auth Setup**

NextAuth needs to know where it's running, and a secret to encrypt sessions with. Set:

```
NEXTAUTH_URL=<where_next_is_running>
NEXTAUTH_SECRET=<some_random_string>
```

For development the default values should be fine, but you can change them if you'd like.

#### **Google Project Setup**

This project relies on Google OAuth2.0 for authentication.
You'll need to create a Google OAuth Client ID and Secret. You can do this by following the instructions [here](https://next-auth.js.org/providers/google).

Once you have these secrets, store them in the `.env.dev` file you created above:

```
GOOGLE_CLIENT_ID=<client_id>
GOOGLE_CLIENT_SECRET=<client_secret>
```

### Running the Development Server

Make sure to create a `.env.dev` file as described above.

Run the core server from the rust project:

```bash
cd .. && cargo run
```

This should set up the database and run the server.

Run the frontend:

```bash
yarn dev
```

This should start the frontend server on port 3000.

<!-- ### Running with Docker

Build a development docker image:

```bash
docker-compose build
```

Run a development docker container:

```bash
docker-compose up
```

If you have a properly configured `.env.dev` file, the frontend will be available at http://localhost:3000.

### Running Dev Server Locally

You can run this project locally without docker, if you prefer, but I'm not going to document that here. You will need to run Postgres locally and point your NextJs app at it, as demonstrated in the `docker-compose.yml` file.
