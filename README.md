# Features
- Handle unlinked torrents (torrents that have no hardlinkes outside the torrent folder)
- Handle unregistered torrents (torrents that have been deleted from the tracker)
- Handle orphaned files & empty folders (stuff that isn't in the torrent client anymore)
- Health check for files
  - Missing torrent contents
  - Torrent contents size is different than the actual file size
  - Files are directories instead of files
- Striking (action only taken on x strikes over y **continuous** days)
- Protection Tag for every feature
- Discord Webhook Notifications
- Never delete files that other torrents need (full cross-seed support ! hardlinks only !)
- Written in Rust with a focus on performance and stability
- Supported torrent clients:
  - qBittorrent

# Prerequirements
- Use hardlinks only! Symlink is not supported/tested and could cause data loss!

# How to install

## Docker Compose
```yaml
torrent-cleaner:
    image: silentesc/torrent-cleaner:stable # Explanation of docker tags below
    container_name: torrent-cleaner
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=Etc/UTC
      - TORRENTS_PATH=/data/torrents
      - LOG_LEVEL=INFO # TRACE, DEBUG, INFO, WARN, ERROR
    volumes:
      - ./config/torrent-cleaner:/config
      - ./data/torrents:/data/torrents
    restart: unless-stopped
```

### Volume mapping and TORRENTS_PATH
When torrent-cleaner communicates with qBittorrent to get paths of torrents, qBittorrent gives the path it knows. For example, if you mount qBittorrent to `./data:/data` and your torrents are in `./data/torrents`, qBittorrent will tell torrent-cleaner `/data/torrents`. torrent-cleaner has to know that. Because if qBittorrent knows `/data/torrents` and torrent-cleaner only knows `/torrents`, it is confused and cannot find any files. Thats what TORRENTS_PATH is for.
#### Examples:
| Your torrents folder | qBittorrent volume mapping | torrent-cleaner volume mapping | TORRENTS_PATH |
| --- | --- | --- | --- |
| ./torrents | ./torrents:/torrents | ./torrents:/torrents | /torrents |
| ./data/torrents | ./data/torrents:/data/torrents | ./data/torrents:/data/torrents | /data/torrents |
| ./data/torrents | ./data:/data | ./data/torrents:/data/torrents | /data/torrents |
| ./torrents/qbittorrent | ./torrents/qbittorrent:/torrents/qbittorrent | ./torrents/qbittorrent:/torrents/qbittorrent | /torrents/qbittorrent |qBittorrent

## Docker Tags
| Tag | Description |
| --- | --- |
| `stable` | Stable release (recommended) |
| `develop` | Untested or beta version |
| version (e.g. `v1.0.0`) | Stable release (e.g. for pinning or switching back) |

## Config
The config will create itself on first start with recommended settings, but still needs to be configured for notifications and the torrent client

### !!! Don't paste the explanation comments in your config, json doesn't like that !!!
```json
{
  "notification": {
    "discord_webhook_url": "", // Leave empty to disable
    "on_job_action": true,
    "on_job_error": true
  },
  "torrent_client": {
    "client": "qbittorrent",
    "base_url": "http://qbittorrent:8080",
    "username": "admin",
    "password": "adminadmin"
  },
  "jobs": {
    "handle_unlinked": {
      "interval_hours": 12, // -1 to disable, 0 to directly start when running (e.g. for testing)
      "min_seeding_days": 20,
      "min_strike_days": 3,
      "required_strikes": 3,
      "protection_tag": "protected-unlinked",
      "action": "test" // test, stop, delete
    },
    "handle_unregistered": {
      "interval_hours": 3, // -1 to disable, 0 to directly start when running (e.g. for testing)
      "min_strike_days": 1,
      "required_strikes": 2,
      "ignore_dht": true,
      "ignore_pex": true,
      "ignore_lsd": true,
      "protection_tag": "protected-unregistered",
      "action": "test" // test, stop, delete
    },
    "handle_orphaned": {
      "interval_hours": 13, // -1 to disable, 0 to directly start when running (e.g. for testing)
      "min_strike_days": 3,
      "required_strikes": 3,
      "protect_external_hardlinks": true,
      "action": "test" // test, delete
    },
    "health_check_files": {
      "interval_hours": 24 // -1 to disable, 0 to directly start when running (e.g. for testing)
    }
  }
}
```
