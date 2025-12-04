# IN ACTIVE DEVELOPMENT

# Features
- Striking (action only taken on x strikes over y **continuous** days)
- Protection Tag individually for every job
- Discord Webhook Notifications
- Supports Cross-Seeding that
  - uses hardlinks
  - uses the excact same files
- Written in Rust with a focus on performance and stability
- Supported torrent clients:
  - qBittorrent

# Jobs
- HandleForgotten (handle torrents that are not present in the media dir):
  - All features from above, plus:
    - Minimum seeding days (action only taken if torrent was **actively seeding** for x days)
- HandleNotWorking (handle torrents that have no working trackers)
  - All features from above, plus:
    - If there is a working tracker, the process is reset
- HandleOrphaned (handle files/folders that are not used by any torrent)
  - All features from above

# Prerequirements
- Have the torrents and media library on the same filesystem (needed for hardlinking)
- Use hardlinks only! Symlinks etc. are not supported and could in worst case cause data loss!
- Have a parent folder with torrents/media folder inside (e.g. /data | /data/torrents | /data/media)

# How to install

### Docker Compose
```yaml
torrent-cleaner:
    image: torrent-cleaner # Will be released on docker hub soon
    container_name: torrent-cleaner
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=Etc/UTC
      - TORRENTS_PATH=/data/torrents
      - MEDIA_PATH=/data/media
      - LOG_LEVEL=INFO # TRACE, DEBUG, INFO, WARN, ERROR
    volumes:
      - ./config/torrent-cleaner:/config
      - ./data:/data
    restart: unless-stopped
    depends_on:
      qbittorrent:
        condition: service_started
```

### Config
The config will create itself on first start with recommended settings, but still needs to be configured for notifications and the torrent client
```json
{
  "notification": {
    "discord_webhook_url": ""
  },
  "torrent_client": {
    "client": "qbittorrent",
    "base_url": "http://qbittorrent:8080",
    "username": "admin",
    "password": "adminadmin"
  },
  "jobs": {
    "handle_forgotten": {
      "interval_hours": 24,
      "min_seeding_days": 20,
      "min_strike_days": 3,
      "required_strikes": 3,
      "protection_tag": "protected",
      "action": "test"
    },
    "handle_not_working": {
      "interval_hours": 3,
      "min_strike_days": 5,
      "required_strikes": 10,
      "protection_tag": "protected",
      "action": "test"
    },
    "handle_orphaned": {
      "interval_hours": 24,
      "min_strike_days": 3,
      "required_strikes": 3,
      "action": "test"
    }
  }
}
```
