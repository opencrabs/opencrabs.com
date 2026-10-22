# Six environments, one autonomous ops layer

<div class="story-meta">

*Telegram* **DevOps · Infra**

</div>

Two products, six environments, one crab keeping them alive: Truelens and Neura/OpenCrabs across dev, staging, and production. It deploys, watches the certificates, pings uptime, self-heals the web server, and verifies the backups, so nothing falls over quietly.

<div class="story-author">

Adolfo Usier · Creator

</div>

- **Six environments:** Truelens dev/staging/prod and Neura/OpenCrabs dev/staging/prod, each on its own server
- **Certificates watched:** an hourly cert check tracks every SSL certificate and alerts before anything expires
- **Uptime watchdogs:** the sites get pinged every five minutes, and an nginx self-heal restarts the web server if it drops
- **Disk and backups:** monitors catch a filling disk or a failed backup before it becomes an outage
- **Deploy and release tracking:** every deploy and GitHub release kept tabs on automatically

An ops layer that never sleeps: the crab watches the certs, the uptime, the disks, and the backups across all six environments, and fixes what it can before anyone notices.
