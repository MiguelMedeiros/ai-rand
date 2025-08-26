# Docker Setup for AI Rand Bot

## Quick Start

### 1. Make sure the `.env` file exists in the project root

The bot will automatically use the `.env` file from the root.

### 2. Run with Docker Compose

```bash
# Build and start the bot
docker-compose up -d

# View logs
docker-compose logs -f ai-rand-bot

# Stop the bot
docker-compose down
```

### 3. Or run with Docker

```bash
# Build the image
docker build -t ai-rand-bot .

# Run the container
docker run -d --name ai-rand-bot ai-rand-bot
```

## Troubleshooting

### View logs
```bash
docker-compose logs -f ai-rand-bot
```

### Restart bot
```bash
docker-compose restart ai-rand-bot
```
