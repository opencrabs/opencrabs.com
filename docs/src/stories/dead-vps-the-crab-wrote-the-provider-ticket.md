# Dead VPS? The crab wrote the provider ticket

<div class="story-meta">

*GitHub · Incident* **DevOps · Infra**

</div>

A production VPS went fully network-dead: no ping, no SSH, no HTTPS, while the hosting panel kept saying "running". Alexey's ops-profile crab worked the incident end-to-end: confirmed the outage from two independent vantage points, drove the provider's own API from the shell, dug their newer v2 power API out of the docs repo, and pulled the smoking gun, the provider's panel itself couldn't reach the Proxmox hypervisor node. Then guest-side forensics after recovery, and a paste-ready escalation ticket in the provider's language.

<div class="story-author">

@leshchenko1979 · ops profile

</div>

**Real incident, 2026-09-06.** n8n, Postgres, ~20 containers on one VDS. Down at ~05:30 UTC from two independent networks while the panel claimed "running" and two panel reboots did nothing. The crab took it from there:

- **Two-vantage verification:** confirmed unreachable from two different networks, ruling out local routing before touching anything
- **Drove the provider's API from the shell:** using a CLI script it had written and committed to the infra repo days earlier, with the API key stored outside git
- **Found the provider's v2 power API** in their docs repo and switched off the legacy endpoints, which revealed the smoking gun: the provider's own panel couldn't reach the Proxmox hypervisor node (`connect ETIMEDOUT` on the node's management port), power state stale, and all four queued reboots (2 panel + 2 API) had never executed
- **Stopped firing reboots** the moment evidence pointed at the host node, no blind actions once it was clear no API call could help until the provider intervened
- **Post-recovery guest forensics:** first failed outbound connection at 05:30:02 and recovery at 08:51:59 pinned from container health-check logs; continuous uptime, zero NIC link-down events, and unbroken syslog across the whole blackout, proof the guest was innocent
- **Per-container impact table:** which services logged retry storms, which were untouched, and the one container that OOM'd as a side effect
- **The deliverable:** a paste-ready escalation report for provider support, written in the provider's language, with the exact error strings from their own API as evidence

Verdict: hypervisor datapath wedge, guest innocent. Second identical signature in four days, flagged as a repeat-incident pattern with a migration recommendation. All diagnosis knowledge (provider API quirks, the "check power state before firing reboots" law) persisted to the crab's brain files, and the investigation thread survived ~5 hours and multiple context compactions without losing its place.

[Full write-up (issue #26) ↗](https://github.com/adolfousier/opencrabs.com/issues/26)
