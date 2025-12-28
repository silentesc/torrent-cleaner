# Features
- Striking (action only taken on x strikes over y **continuous** days)
- Protection Tag individually for every job
- Discord Webhook Notifications (disable by leaving it blank)
- Supports cross-seeding that either uses hardlinks or uses the excact same files
- Written in Rust with a focus on performance and stability
- Supported torrent clients:
  - qBittorrent

# Jobs
- HandleUnlinked (handle torrents that have no hardlinkes outside the torrent folder):
  - All features from above, plus:
    - Minimum seeding days (action only taken if torrent was **actively seeding** for x days)
  - Supported actions:
    - test (Log, Discord Notification)
    - stop (Stop torrent, Log, Discord Notification)
    - delete (Delete torrent (and files if possible), Log, Discord Notification)
- HandleNotWorking (handle torrents that have no working trackers)
  - All features from above, plus:
    - If there is a working tracker, the striking process is reset
  - Supported actions:
    - test (Log, Discord Notification)
    - stop (Stop torrent, Log, Discord Notification)
    - delete (Delete torrent (and files if possible), Log, Discord Notification)
- HandleOrphaned (handle files/folders that are not used by any torrent, no matter if they are also in the media folder)
  - All features from above
  - Supported actions:
      - test (Log, Discord Notification)
      - delete (Delete files/folders, Log, Discord Notification)

# Prerequirements
- Use hardlinks only! Symlink, copying files, etc. is not supported and could cause data loss!

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
    "discord_webhook_url": "" // Leave empty to disable
  },
  "torrent_client": {
    "client": "qbittorrent",
    "base_url": "http://qbittorrent:8080",
    "username": "admin",
    "password": "adminadmin"
  },
  "jobs": {
    "handle_unlinked": {
      "interval_hours": 20, // -1 to disable, 0 to directly start when running (e.g. for testing)
      "min_seeding_days": 20,
      "min_strike_days": 3,
      "required_strikes": 3,
      "protection_tag": "protected-unlinked",
      "action": "test" // test, stop, delete
    },
    "handle_not_working": {
      "interval_hours": 3, // -1 to disable, 0 to directly start when running (e.g. for testing)
      "min_strike_days": 5,
      "required_strikes": 10,
      "protection_tag": "protected-not_working",
      "ignore_dht": true,
      "ignore_pex": true,
      "ignore_lsd": true,
      "action": "test" // test, stop, delete
    },
    "handle_orphaned": {
      "interval_hours": 20, // -1 to disable, 0 to directly start when running (e.g. for testing)
      "min_strike_days": 3,
      "required_strikes": 3,
      "action": "test" // test, delete
    }
  }
}
```
