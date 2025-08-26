## AI Rand

AI Rand is a Rust bot for the Pubky network that automatically replies to mentions. It:

- Signs in to Pubky using a BIP39 mnemonic
- Creates/updates a minimal profile
- Polls notifications from a Nexus aggregator
- Fetches the original post content
- Generates a concise reply using OpenAI (model: gpt-4o-mini), grounded by a local knowledge base
- Publishes the reply back to Pubky and advances `last_read`

### How it works (high level)
- Sign-in: derives a `pubky` `Keypair` from `BOT_SECRET_KEY` (BIP39 mnemonic) and verifies it matches `BOT_PUBLIC_KEY`.
- Profile: writes `pub/pubky.app/profile.json` to your homeserver.
- Notifications: polls `NEXT_PUBLIC_NEXUS` for `mention` notifications newer than `last_read`.
- Content: resolves the mentioned post via `pubky://...` and extracts text.
- Response: calls OpenAI Chat Completions with a system prompt plus `knowledge-base.txt` context, enforcing ≤1000 chars.
- Publish: writes a reply post to `pub/pubky.app/posts/<timestamp>` and updates `pub/pubky.app/last_read`.

### Repository structure
- `src/main.rs`: bot logic (sign-in, polling, replying)- `knowledge-base.txt`: compact knowledge base included in the system prompt
- `.env-sample`: environment variable template
- `Cargo.toml`: Rust package configuration

### Prerequisites
- Rust (latest stable recommended)
- An OpenAI API key with access to `gpt-4o-mini`
- A Pubky identity (public key) and BIP39 mnemonic seed usable with `pubky`
- A reachable Nexus aggregator URL (e.g., local dev, testnet, or mainnet)

### Quick start
1) Copy and edit environment variables
```bash
cp .env-sample .env
# Fill the blanks in .env
```

2) Build and run
```bash
cargo run
```

On first run the app will:
- Load `.env`
- Sign in to Pubky
- Write/update your profile
- Start polling notifications every 5 seconds

### Configuration (.env)
- `BOT_PUBLIC_KEY` (required): The bot’s Pubky public key string.
- `BOT_SECRET_KEY` (required): BIP39 mnemonic words used to derive the secret key. Must produce `BOT_PUBLIC_KEY`.
- `OPENAI_API_KEY` (required): OpenAI API key.
- `NEXT_PUBLIC_NEXUS` (required): Nexus aggregator base URL, e.g. `http://localhost:8080`.
- `TESTNET` (optional): `true` to use Pubky testnet client configuration, otherwise mainnet. Default: `false`.
- `HOMESERVER` (optional): Present for reference; not directly read by the current code path.

Notes:
- The app verifies that the derived public key from `BOT_SECRET_KEY` matches `BOT_PUBLIC_KEY` and exits if they differ.
- `last_read` is stored at `pub/pubky.app/last_read` as `{ "timestamp": <i64> }`. The bot updates it after processing notifications.

### Customization
- Knowledge base: edit `knowledge-base.txt` to adjust the assistant’s context and tone.
- Poll interval: change the `tokio::time::sleep` duration in `src/main.rs` (default 5s).
- Model/constraints: update `model`, `temperature`, or the system prompt in `generate_response` inside `src/main.rs`.

### Troubleshooting
- Missing env vars: the app logs explicit errors, e.g. `OPENAI_API_KEY not found in .env`.
- Public key mismatch: ensure `BOT_PUBLIC_KEY` corresponds to the mnemonic in `BOT_SECRET_KEY`.
- OpenAI errors: check API key, network, and model access; the app logs the raw response body for diagnosis.
- Empty/invalid `last_read`: if absent on first run, ensure your homeserver allows writing it; the bot updates it after processing.
- Nexus connectivity: confirm `NEXT_PUBLIC_NEXUS` is reachable and returns notifications for the bot user.

### Docker

To run with Docker:

```bash
# Build and start
docker-compose up -d

# Logs
docker-compose logs -f ai-rand-bot
```

See `DOCKER.md` for more details.

### Security
- Never commit real secrets. Keep `.env` local.
- Rotate `OPENAI_API_KEY` and regenerate mnemonics if leaked.

### License
No license file is present. If you plan to distribute or modify, consider adding a license.

