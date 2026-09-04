### Docker

You can see the [server example](./self-hosting-example/) for reference.

First, you need to set the required environment variables.

You can generate a `PRIVATE_KEY` using the following command:

```sh
docker run --rm heraclitoqsaldanha/harmon-server harmon-server generate-key
```

You can also generate the `JWT_SECRET` and `JWT_CHALLENGE_SECRET` using the following command:

```sh
docker run --rm alpine/openssl rand -base64 32
```

After generating the keys and secrets, make sure to set the corresponding environment variables in your `.env` file.

Then, you can start the server with:

```sh
docker compose up -d
```
