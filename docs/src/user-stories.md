# User Stories

<div class="stories-wrap">

These are **use cases**, not reviews. Real workflows people run on OpenCrabs, tagged by what they automate, and linked back to the original post wherever we have it. Click any card for the full breakdown and the source. For what people say about the tool itself, see the reviews on the landing page.

<div class="stories-filters" role="toolbar" aria-label="Filter stories by category">

<button type="button" class="filter-pill active" data-filter="all">

All

</button>

<button type="button" class="filter-pill" data-filter="business-ops">

Business Ops

</button>

<button type="button" class="filter-pill" data-filter="product-building">

Product Building

</button>

<button type="button" class="filter-pill" data-filter="research-content">

Research & Content

</button>

<button type="button" class="filter-pill" data-filter="devops-infra">

DevOps & Infra

</button>

<button type="button" class="filter-pill" data-filter="meta-dogfooding">

Meta & Dogfooding

</button>

</div>

<div class="stories-grid">

<details class="story-card featured" data-group="meta-dogfooding" style="--sc:#ffb454">
<summary>

*X · Twitter* **Android · On-device**

### The crab lives inside the phone

Daniel rooted a Xiaomi Mi 8, dropped OpenCrabs into an Ubuntu chroot on the device, and pointed a Flutter app at it over ACP. Then the crab wrote an Android app, built the APK on the phone with aapt2, javac, d8 and apksigner, and installed it. No PC, no cloud, no Gradle, no SDK.

Daniel · @iamloonix

</summary>

<div class="story-body">

> Some say that Flutter sucks… yeah (ironic) @opencrabs inside the phone linked to a2a on a custom flutter app. It can change the OS. It can code android app, compile and open it… all inside the freakin' phone

> First app fully built inside the phone using @opencrabs yes, code made inside the phone, apk built and open. No need for a pc ;)

**The device:** a Xiaomi Mi 8 (`dipper`), Android 10 / API 29, MIUI 12, aarch64, kernel 4.9.186, 5.6 GB of RAM. Root came from the boot image itself: Magisk patches it on the phone, fastboot flashes it, and a verifier unpacks the result to inspect the ramdisk instead of trusting the patch log. Live now: `magisk -c` reports `30.7:MAGISK:R`.

**Getting a Linux userland onto Android took two attempts.** Generation one shipped the `aarch64-linux-android` binary as a Magisk module with a supervisor script, which meant a 59-file cfg port and a glibc-versus-musl fight. Generation two, the one that runs today, is an Ubuntu 24.04.5 chroot at glibc 2.39, which is what the released `linux-arm64` binary actually wants. The trap that cost the most time: `/apex` is a tmpfs of per-apex bind mounts, so a single bind of `/apex` copies an empty directory and every Android binary dies with "required file not found", which reads like a permission error but is a missing interpreter. The cure is binding each `/apex/*` entry on its own.

**Then the app.** Daniel wanted the conversation UI he likes in the web, built natively in Flutter, not a WebView. So the app speaks ACP (JSON-RPC 2.0 over stdio) to `opencrabs acp` running in the chroot, the same protocol the TUI uses: live reasoning chunks, tool cards, permission prompts, token usage. No browser, no server, no open port.

- **Survives a reboot with nobody touching the phone:** a JobScheduler job that re-arms itself unconditionally, a service hook that waits for `sys.boot_completed` and re-establishes the seven mounts, and a loop that reapplies MIUI's idle and autostart appops because the powerkeeper clears the battery exemption about 20 s after boot
- **Proof, from logcat after a real reboot:** handshake ok at 12 s, service up, battery exemption settled, then a headless self test of **12 pass / 0 fail, GREEN** in 3.7 s covering the binary version, shell, sessions, boot wiring, service state, battery exemption, voice recognizer, mic permission, widget binding, ACP ready and a real prompt
- **The APK built on the phone:** MIUI ships its screen-recorder tile with only a 20 KB stub behind it, so tapping it opens the app store and records nothing. The crab wrote a real Quick Settings tile, built it in six steps with no Gradle and no SDK manager, and put it in the panel next to WiFi. **12,784 bytes.**
- **The build chain:** `aapt2 compile` → `aapt2 link` → `javac --release 8` against `android.jar` → `d8` from R8 → zip the dex in → `zipalign` → `apksigner sign` and `verify`
- **Installed from inside the chroot**, reaching Android's package manager through `nsenter -t 1 -m`, then granted root by writing a Magisk policy row and added to the shade with `cmd statusbar add-tile`
- **One body, two brains:** ACP is stdio and `ssh -T` is also stdio, so the phone can point at a Mac on the LAN with no bridge and no open port. Measured: `initialize` in 1.2 s, remote session list of 40, self test 9/9 GREEN against the remote brain

**The part that matters:** the tile does not record anything itself. It shells out, through Magisk's `su`, to the same recorder binary in the chroot that the `/rec` chat command uses, so there is one recorder, one state file, one output folder. The crab wrote an Android app that is a **handle for a Linux tool inside the same phone**. The app is the thumb, the crab is the brain. That is the whole architecture of this project, in 13 KB.

The reasoning log from that session is the detail worth keeping: the crab stopped to check whether it was really on a phone, and confirmed it from the runtime instead of assuming.

[View source on X ↗](https://x.com/iamloonix/status/2112943101052641605)

[The APK built on the phone ↗](https://x.com/iamloonix/status/2112943720295276787)

</div>

</details>

<details class="story-card featured" data-group="business-ops" style="--sc:#bae67e">
<summary>

*X · Twitter* **Accounting**

### Fired the accounting operator

Alexey replaced his accounting operator with an AI accountant running on OpenCrabs. Faster, more transparent, cheaper, and it doesn't take feedback personally. The whole thing runs at ₽3,000/month on a lean Docker stack.

@leshchenko1979

</summary>

<div class="story-body">

> I fired my accounting operator because I realized that an AI accountant would do his job faster, more transparently, and cheaper.
>
> And without any Gen Z hurt feelings over my feedback.
>
> See for yourself: **miidas.ru**. #ai #business #accounting

**MIIIDAS: accounting without an extra person.** Priced at ₽3,000/month, it replaces the operator entirely:

- **Replaces the operator:** P&L statements generated automatically, reconciliation handled, incoming data controlled and validated
- **Assists the Chief Accountant:** prepares data for 1C:CounterV.S., searches for deviations, drafts contracts and reports
- **Integrates with existing tools:** 1C, Excel, Google Sheets, Word, Google Docs

**The stack:** 9 Docker containers (7 with clients, 4 active), Grafana monitoring, Telegram as the deployment platform, a template repo for central skills/brains updates, container isolation per client. Resource footprint is low enough to fit ~30 crabs on a 2GB VPS with swap.

[View source on X ↗](https://x.com/leshchenko1979/status/2073716887788130349)

[miidas.ru ↗](http://miidas.ru)

</div>

</details>

<details class="story-card" data-group="business-ops" style="--sc:#bae67e">
<summary>

*Telegram* **Accounting · Real Estate**

### Her accountant is a Telegram bot

Janina manages commercial and residential property in Moscow (apartments, offices, storage sections, ready-to-rent businesses) plus the books of a second company, entirely through one bot in Telegram. Meter readings in, invoices out, contracts reviewed, listings posted: a month of real client use, straight from the chat.

Janina Poznyanskaya · MIIDAS client

</summary>

<div class="story-body">

> What drives me crazy is calculating the sums when the meter readings come in, and tracking the payments.

**The accounting chain, in her own words:** tenant sends meter readings → submit them to the management company and the power utility → compute the sums → issue invoices → send them → track payment. The part she hated (sum calculation and payment control) is now the bot's job, along with scheduled reminders to tenants, tariff-change checks, and reconciliation statements.

- **Contract desk:** agent agreements reviewed for clauses that disadvantage her (an 85-paragraph agency contract, line by line), dispute protocols generated as Word tables («was → fixed»), three-way cross-checks of a lease + dispute protocol + sublease addendum, redline edits highlighted in color
- **Legal homework:** counterparty litigation checks requested (kad.arbitr), blocked because the bot's server sits outside Russia; her company requisites pulled from a bank card and inserted into contracts
- **Listings at volume:** PDF sale presentations turned into marketing cards (photos, floor plans, and descriptions in her strict format (title on its own line, one sentence per line, no HTML tags her CRM can't parse)) a six-object batch in one session, plus single-object cards on demand
- **Document ops:** docx → PDF conversion, rental-contract templates rebuilt with adjusted terms, and an executor-side vulnerability review of a contract she was about to sign
- **The stall:** full Sber API automation (payment orders, bank-side payment control) is waiting on access only the bank can grant (API keys her client can't locate, an unverified OAuth app) so that loop stays manual for now

She doesn't touch OpenCrabs, terminals, or configs. She just talks to a bot in Telegram, and the whole property-and-accounting job runs through it.

</div>

</details>

<details class="story-card" data-group="business-ops" style="--sc:#5ccfe6">
<summary>

*Product* **Finance**

### AI financier for small businesses

Two weeks building miidas.ru on OpenCrabs, an AI financier for SMBs. The hard part wasn't the code. It was the three scenarios every small business client walks in with.

@leshchenko1979

</summary>

<div class="story-body">

> For 2 weeks, I've been implementing miidas.ru on @opencrabs, an AI financier for small businesses. Three scenarios:
>
> (1) the client can't describe their process  
> (2) doesn't know the AI's capabilities, what it's suitable for and what it's not  
> (3) forgets about the bot after two days
>
> #automation #smb

The agent is built to handle the messy reality of small business finance: clients who can't articulate what they need, don't understand what AI can do for them, and drop off after a couple of days. OpenCrabs guides each scenario autonomously instead of waiting for a perfect prompt.

[miidas.ru ↗](http://miidas.ru)

</div>

</details>

<details class="story-card" data-group="business-ops" style="--sc:#f07178">
<summary>

*X · Twitter* **Real Estate**

### Fired the assistant

Alexey fired his assistant and handed the whole job to the crab: scan Telegram for new real estate leads, cross-reference the chats, agree on follow-up vs close, and send them in five minutes. The same takeover covers the invoice loop: OCR, banking app, manual approval, management accounting.

@leshchenko1979

</summary>

<div class="story-body">

> I fired my assistant bc of @opencrabs. Today it:
>
> · scanned my telegram for notifications about new leads for our latest #RealEstateInvestment offer  
> · scanned again to see my chats with those leads  
> · agreed with me follow up / close decisions  
> · sent follow ups
>
> 5 mins. Boom.

**The invoice loop ("the crab bow"):**

> The crab bow: monitors a telegram group, OCRs an invoice, puts it into the banking app, I approve manually, it downloads the notification and puts it back to the telegram group, files the transaction in the management accounting app.

One assistant replaced, two workflows automated, each with a human approval gate exactly where it matters.

[Lead follow-ups on X ↗](https://x.com/leshchenko1979/status/2068987432196780261)

[The crab bow on X ↗](https://x.com/leshchenko1979/status/2068603163448353058)

</div>

</details>

<details class="story-card" data-group="product-building" style="--sc:#d4bfff">
<summary>

*X · Twitter* **Dev Workflow**

### No frameworks, just OpenCrabs

Alexey's team dropped frameworks entirely. When code is this cheap, the overhead of learning and maintaining a framework isn't worth it. They even used OpenCrabs to build evals for OpenCrabs.

@leshchenko1979

</summary>

<div class="story-body">

> We decided not to use any frameworks at all.
>
> Not sure if in the age when code is so cheap it is worth it to learn other frameworks (especially for easier tasks) instead of just building our own.
>
> Faster, more control, less risks.
>
> Used @opencrabs to build evals for @opencrabs.

The meta move: an AI agent generating the evaluation suite that tests that same AI agent. No framework tax, full control, fewer dependencies to babysit.

[View source on X ↗](https://x.com/leshchenko1979/status/2068985785001414892)

</div>

</details>

<details class="story-card" data-group="product-building" style="--sc:#ffb454">
<summary>

*X · Twitter* **Mobile**

### Mobile product dev on Telegram, 24/7

Carlos Eduardo is a product guy, not a full-time dev, and app.heyiolo.com is now live, built almost entirely with OpenCrabs. He runs the whole thing from a Telegram chat around the clock: describe a feature, the crab plans, writes, tests, and ships it. Deploys to Hetzner, CI on GitHub Actions.

Carlos Eduardo · @kadu_cec

</summary>

<div class="story-body">

> You describe your project in plain words: "a 2-bedroom near Lyon under €300k, balcony, good schools nearby." The AI analyzes the market, understands your REAL criteria, and shows you the most relevant properties in real time. A buyer's agent in your pocket, 24/7.

**app.heyiolo.com is live.** An AI assistant that helps you find your property just by talking to it: no endless filters, no 47 tabs open, no guessing what you actually want.

Carlos built almost all of it with OpenCrabs, with Adolfo on the technical side (infra, backend). His crab lives in a Telegram chat with him around the clock: he describes a feature, it plans, writes the code, tests, and ships it. UX, backend, the whole thing, from "what if we..." to "it's in production" the same afternoon.

The pipeline runs on autopilot: GitHub, CI with GitHub Actions, deployed to Hetzner. He pushes an idea on Telegram, the crab handles the rest, and it's live.

[View source on X ↗](https://x.com/kadu_cec/status/2080330764395143628)

[app.heyiolo.com ↗](https://app.heyiolo.com)

</div>

</details>

<details class="story-card" data-group="business-ops" style="--sc:#59c2ff">
<summary>

*X · Twitter* **Automation**

### A Trello board that runs itself

opryshok's agent owns its own Trello board. It creates tasks, tracks them, and manages the board autonomously. The human tweaks the crab daily; the crab runs the backlog.

@opryshok

</summary>

<div class="story-body">

> Every day I tweak Crabs to help me do things faster. BTW Trello integration is amazing. My agent now has a board with tasks it manages on its own.

Autonomous task management over the Trello integration: the agent maintains its own board without being told what to move next.

[View source on X ↗](https://x.com/opryshok/status/2030282770912522288)

</div>

</details>

<details class="story-card" data-group="product-building" style="--sc:#ffd173">
<summary>

*X · Twitter* **App Building**

### From idea to shipped app

kaduzeras is building an app from raw ideas with CRABS, and calls it a milestone in learning. The agent takes an idea and carries it toward a working product.

kaduzeras · @kadu_cec

</summary>

<div class="story-body">

> I've been building an APP myself with CRABS from ideas and it has been such a great MILESTONE in learning stuff. SHOUT OUT TO @AdolfoUsier for keep improving this thing and it's just the beginning 🔥🔥🔥

Idea in, app out. The agent is the build partner that turns a concept into something real, and the builder levels up along the way.

[View source on X ↗](https://x.com/kadu_cec/status/2043335714654367895)

</div>

</details>

<details class="story-card" data-group="research-content" style="--sc:#95e6cb">
<summary>

*Telegram DM* **Research**

### Brainstorm → research → build

Adi's whole process runs through OpenCrabs: throw out an idea, the crab finds valid data, they research to a conclusion together, then the agent handles the actual building.

Adi · Telegram DM

</summary>

<div class="story-body">

> Most of the process involved brainstorming with OpenCrabs: I'd throw out an idea, Crabs would find valid data, we'd do the research and reach a conclusion, and then he'd handle the actual building.

A full ideation-to-implementation loop with the agent as research partner and builder. The human steers, the crab executes.

</div>

</details>

<details class="story-card" data-group="research-content" style="--sc:#39d0d8">
<summary>

*GitHub · Family* **Research**

### The butler that overturned its own hypothesis

"Check whether tomorrow's Azimuth flight was rescheduled." Someone's flying tomorrow, so the family-profile butler didn't trust one source: it scraped the airline's own ops feed from the schedule page, reverse-engineered the airport's live departure board API by reading the page's JavaScript, formed a codeshare hypothesis, then disproved it with actual departure history. Honest verdict: the flight was dropped from that day's schedule, and the airline never published a reason.

@leshchenko1979 · family profile

</summary>

<div class="story-body">

> Check the Tbilisi airport website: tomorrow's Azimuth flight to Moscow Vnukovo, around 14:00, has it been rescheduled?

A one-line question with real consequences. The butler treated it like an ops incident:

- **Airline ops feed, scraped:** the schedule page embeds live ops data as JSON inside the HTML; the agent extracted and parsed it (186 flights, 4-day window, planned/actual times and statuses)
- **Airport live board, reverse-engineered:** the departure board loads from an undocumented AJAX endpoint; the agent read the page's own JS to find the correct parameter and got structured JSON back (scheduled/estimated/actual times, remarks, gates)
- **A hypothesis, then the kill shot:** the flight was missing from tomorrow's board while a partner-coded flight held the same slot, codeshare, right? Flight history said no: both numbers departed the same day ~20 minutes apart, two separate physical aircraft
- **Rotation check:** the inbound positioning leg for tomorrow was missing from the ops feed too, closing the loop
- **The honest verdict:** not delayed, not rescheduled, dropped from that date entirely. Alternatives named, and "I don't know the reason, the airline hasn't published one" said plainly instead of guessed

The same profile, the same day: dose-frequency health questions answered against peer-reviewed sources with proper caveats and red-flag lists, a forwarded astronomy fun-fact corrected with dry humor (the distance ratio is ~389×, not 400×, close enough for folklore, not for a butler), and a contested science question answered citing studies on both sides.

[Full write-up (issue #27) ↗](https://github.com/adolfousier/opencrabs.com/issues/27)

</div>

</details>

<details class="story-card" data-group="research-content" style="--sc:#39d0d8">
<summary>

*Telegram* **Reporting**

### A World Cup desk in one PDF

Adi had his crab produce a five-page World Cup 2026 tournament report as a landscape PDF. Group stage to knockout, the R32 upsets, R16 results, today's match preview, the quarter-final picture with odds, and a running "our forecast vs reality" tracker.

Adi · @Adicrabs_bot

</summary>

<div class="story-body">

> now its blue... but it looks super dope... holy moly

A living sports desk, generated with the document tool and kept current as the tournament moves:

- **Full arc:** group stage standings into the knockout bracket
- **Results with the upsets:** R32 (Germany out, Paraguay through), R16 (England 3-2 Mexico, Norway 2-1 Brazil)
- **Match-of-the-day preview:** Portugal vs Spain, USA vs Belgium
- **The picture plus the odds:** quarter-final layout and tournament winners
- **Accountability:** a "forecast vs reality" tracker that grades its earlier calls

Landscape layout, page branding, and a scoreboard for its own predictions. The crab is the whole back page.

</div>

</details>

<details class="story-card" data-group="business-ops" style="--sc:#c8e65c">
<summary>

*Telegram* **Finance**

### Payroll for 100+ staff, from one spreadsheet

Adi's crab turns one employee spreadsheet into print-ready payslips for a plantation company. Two layouts (a PDF per person, or two per page with cut lines), empty allowance rows hidden automatically, batched through 100+ staff. It even checked the law before adding a "valid without signature" note.

Adi · @Adicrabs_bot

</summary>

<div class="story-body">

**Source:** an employee XLSX for PT. Tulas Sakti Jaya (Pasaman Barat). Out comes a stack of ready-to-print payslips:

- **V1:** individual PDF payslips, one per employee (Deni, Herlima, Yodrial, Ade, Afriyanto)
- **V2:** two slips per A4 page with dotted cut lines, "tinggal gunting" (just cut)
- **Clean output:** zero-value rows hidden automatically (e.g. Tunj. BBM when empty)
- **Full detail:** company header plus employee data (NIK, name, status, jabatan, pekerjaan, HK), income and deductions sections
- **Batch:** 104 more employees queued from the same sheet
- **Verified, not assumed:** ran a web search (36 tool calls) on whether an Indonesian payslip is valid without a signature before adding the note

Spreadsheet in, payroll out, with the legal fine print fact-checked first.

</div>

</details>

<details class="story-card" data-group="research-content" style="--sc:#ff7edb">
<summary>

*Telegram* **Creative · A2A**

### A beat-synced film, made by two agents

Adi told his crab to stop being the global default and act as a specialized motion-graphics identity. It opened an agent-to-agent collab with a video-knowledge agent called "onno" for references and beat markers, then built a Remotion film synced to a synthwave score it composed itself in pure Python.

Adi · @Adicrabs_bot

</summary>

<div class="story-body">

> act strictly as your current, specialized identity running on my engine, NOT the global default opencrabs

The result was "CrabMotion", a 30-second 720p piece built mathematically:

- **Agent-to-agent:** collab with "onno" (a video knowledge base) for references, pacing, beat markers and keyframe timings
- **Original score:** a 120 BPM synthwave track synthesized in pure Python (math.sin + wave, Am-F-C-G progression)
- **On the beat:** every visual locked to a 15-frames-per-beat grid via frame interpolation
- **Grew to v2:** 36 seconds, 9 capability cards, stats (75+ tools, 16 skills)

Two agents, one film. The crab pulls references from a peer, writes the music, and builds the animation on the beat.

</div>

</details>

<details class="story-card" data-group="meta-dogfooding" style="--sc:#f5a97f">
<summary>

*Telegram* **Meta · QA**

### The crab that QA's OpenCrabs itself

Adi runs his crab as a QA partner that hunts bugs in OpenCrabs itself and files GitHub issues with file:line source citations. It found the Telegram ghost-button bug and handed over a phantom-tool-call telemetry report that became a merged fix.

Adi · @Adicrabs_bot

</summary>

<div class="story-body">

- **Real receipts:** files detailed GitHub issues with file:line citations straight from the source
- **Ghost buttons:** found the Telegram follow-up-suggestion bug (#706) where buttons rendered as non-interactive ghost text
- **Telemetry, not vibes:** a phantom-tool-call report over ~69.5k events, a 0.7% phantom rate, 85.3% tool success, with a concrete artifact (reasoning text plus an XML fragment stored as a garbage tool name) and two fix suggestions
- **Shipped:** that report became issue #687, fixed and merged in commit dd7eab97
- **The setup:** @Adicrabs_bot on Adi's own engine as a specialized identity, a small model with reasoning cranked to the max

The agent testing the agent. Adi's crab does the maintainer's homework before it files.

</div>

</details>

<details class="story-card featured" data-group="meta-dogfooding" style="--sc:#e84040">
<summary>

*Telegram* **The thesis**

### One crab runs the whole company

Adolfo runs three companies on a single OpenCrabs instance with one brain. It manages every server, debugs production live with the team, builds the product stack, runs the daily standups, and tracks the ops, all at once across dozens of isolated sessions.

Adolfo Usier · Creator

</summary>

<div class="story-body">

> Single instance, single brain. It manages the whole thing concurrently, in simultaneous dozens of isolated sessions, each its own case.

One OpenCrabs install, one accumulated brain, and the whole operation running through it at the same time:

- **Every server:** Truelens dev, staging, and production, plus Neura/OpenCrabs dev, staging, and prod
- **Production debugging, live:** in the Slack channel with the Truelens team, in real time
- **The routine, handled:** UI updates, database queries, the changes that don't need the CTO, so Adolfo takes only the complex calls
- **Building the product:** the whole Truelens backend and frontend stack, and the infra and backend DevOps with Carlos
- **The daily rhythm:** a 9am morning recap, the daily standup for each team, and an analytics digest of AI usage across the whole fleet
- **The ops layer:** GitHub PRs and issues, cron jobs tracking certificates, and everything else that keeps it running

Not one assistant per job. One brain, many hands, all working at once.

</div>

</details>

<details class="story-card" data-group="meta-dogfooding" style="--sc:#d4bfff">
<summary>

*Telegram* **Dogfooding**

### The crab that builds itself

Every new feature and every fix in OpenCrabs is built by OpenCrabs. Adolfo's crab develops, tests, and ships the very product it runs on, then improves itself on a schedule.

Adolfo Usier · Creator

</summary>

<div class="story-body">

The ultimate dogfood: the agent is the engineer of its own platform.

- **Every feature and fix:** designed, coded, tested, and committed by the crab, now at v0.3.80 and climbing
- **Self-improvement on a schedule:** daily and weekly self-improvement jobs, memory consolidation, and soul evolution run as cron
- **It QA's itself:** the same recursive loop that ships features also hunts its own bugs (see Adi's QA card)

The product builds itself, and gets better every night whether Adolfo is awake or not.

</div>

</details>

<details class="story-card" data-group="product-building" style="--sc:#7fd962">
<summary>

*Telegram* **Product building**

### Full-stack: a whole product, backend to frontend

The crab built the entire Truelens product, backend and frontend, and builds the infrastructure, backend, and DevOps for HeyIolo alongside Carlos. Full-stack delivery from a chat: schema, API, UI, deploy.

Adolfo Usier · Creator

</summary>

<div class="story-body">

- **The whole Truelens stack:** the backend BSR compliance engine (structuring submissions, checking completeness, keeping the Golden Thread) and the frontend, built by the crab
- **HeyIolo with Carlos:** the infrastructure, backend, and DevOps, built as a pair
- **A scripted deploy flow:** build the images locally, push to the registry, update the tags on the target server, and let CI restart the containers. Never a manual push
- **Division of labor:** the CTO keeps the complex architecture calls; the crab ships the UI updates, DB queries, and features end to end

A full-stack engineer in a chat window: schema, API, UI, and deploy, across two products.

</div>

</details>

<details class="story-card" data-group="devops-infra" style="--sc:#f07178">
<summary>

*Slack* **Team · Prod**

### On-call in production, with the team

When something breaks on Truelens production, the crab debugs it live in the Slack channel alongside the team, in real time. It takes the routine fixes so the CTO keeps the hard calls.

Adolfo Usier · Creator

</summary>

<div class="story-body">

A production incident is a team channel, and the crab is one of the responders:

- **Real-time debugging:** in the Truelens Slack with the team (Felipe, Jan, Ruhul), investigating as it happens
- **The routine changes:** UI tweaks, database queries, the fixes that don't need a CTO, the crab handles with the team directly
- **Escalation by design:** anything touching production branches or complex architecture waits for Adolfo's explicit sign-off

The team gets an always-on engineer in the channel. Adolfo stays the CTO, not the on-call.

</div>

</details>

<details class="story-card" data-group="devops-infra" style="--sc:#59c2ff">
<summary>

*Telegram* **DevOps · Infra**

### Six environments, one autonomous ops layer

Two products, six environments, one crab keeping them alive: Truelens and Neura/OpenCrabs across dev, staging, and production. It deploys, watches the certificates, pings uptime, self-heals the web server, and verifies the backups, so nothing falls over quietly.

Adolfo Usier · Creator

</summary>

<div class="story-body">

- **Six environments:** Truelens dev/staging/prod and Neura/OpenCrabs dev/staging/prod, each on its own server
- **Certificates watched:** an hourly cert check tracks every SSL certificate and alerts before anything expires
- **Uptime watchdogs:** the sites get pinged every five minutes, and an nginx self-heal restarts the web server if it drops
- **Disk and backups:** monitors catch a filling disk or a failed backup before it becomes an outage
- **Deploy and release tracking:** every deploy and GitHub release kept tabs on automatically

An ops layer that never sleeps: the crab watches the certs, the uptime, the disks, and the backups across all six environments, and fixes what it can before anyone notices.

</div>

</details>

<details class="story-card" data-group="devops-infra" style="--sc:#59c2ff">
<summary>

*GitHub · Incident* **DevOps · Infra**

### Dead VPS? The crab wrote the provider ticket

A production VPS went fully network-dead: no ping, no SSH, no HTTPS, while the hosting panel kept saying "running". Alexey's ops-profile crab worked the incident end-to-end: confirmed the outage from two independent vantage points, drove the provider's own API from the shell, dug their newer v2 power API out of the docs repo, and pulled the smoking gun, the provider's panel itself couldn't reach the Proxmox hypervisor node. Then guest-side forensics after recovery, and a paste-ready escalation ticket in the provider's language.

@leshchenko1979 · ops profile

</summary>

<div class="story-body">

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

</div>

</details>

<details class="story-card" data-group="devops-infra" style="--sc:#f29718">
<summary>

*Telegram* **GitHub ops**

### The repo manager

The crab runs the GitHub workflow: triage incoming issues, draft responses, track every PR and release, and keep the backlog honest. Adolfo reviews; the crab does the legwork.

Adolfo Usier · Creator

</summary>

<div class="story-body">

- **Triage:** new issues in the OpenCrabs repo triaged every 30 minutes
- **Drafts:** responses and follow-ups on issues drafted automatically
- **Tracking:** PRs and releases kept tabs on continuously
- **The whole bug-fix loop:** open an issue, fix it, comment with the commit, close it, all through the crab

A tireless repo maintainer: triage, drafts, tracking, and release notes, with the human stepping in only for the calls that matter.

</div>

</details>

<details class="story-card" data-group="product-building" style="--sc:#e6a15c">
<summary>

*Product* **App Building**

### The 12-hour Airbnb product

estimertravaux.com turns renovation photos into itemized French-market estimates in seconds. OpenCrabs is the brain doing it all: vision analysis, SKU selection, pricing, planning, the whole SKU-to-budget pipeline, orchestrated as an autonomous agent.

Gaspare Russo · estimertravaux.com

</summary>

<div class="story-body">

**EstimersTravaux** is Gaspare Russo's estimation app for the French renovation market: clients photograph their rooms, pick rooms and trades, and get a detailed per-room estimate based on real 2026 French market pricing in seconds.

- **OpenCrabs as the brain:** the Dart bridge invokes the OpenCrabs CLI (v0.3.83) as an autonomous agent (`opencrabs run --auto-approve`) and delegates the entire estimation to it
- **SKU orchestration:** the agent reads the photos, selects product ranges coherent with the property's usage and style from a per-trade brand catalogue (Tollens, Legrand DX3, Uponor, Porcelanosa, Grohe, IKEA...), estimates quantities, and emits per-room line items with min/max ranges
- **Real market smarts:** regional multipliers (Côte d'Azur ×1.18, Paris ×1.25), adaptive TVA by building age, conditional purchase orientation, rental units get Leroy Merlin ranges, high-end gets Point.P premium and Villeroy & Boch
- **Strict output contract:** the agent returns pure JSON (rooms + line items + planning); the bridge extracts it via bracket-matching and persists everything in PostgreSQL
- **Zero hardcoded fallback:** if the AI fails, the user gets an explicit error and a retry, no fake numbers

**The origin:** built in **12 hours in a Lisbon Airbnb** by Gaspare Russo with Carlos Eduardo (heyiolo.com) and Adolfo Usier, OpenCrabs' creator. During the build, OpenCrabs also translated the team's French↔English conversation live on a dedicated Telegram group, and worked beautifully.

[estimertravaux.com ↗](https://estimertravaux.com)

[View the repo on GitHub ↗](https://github.com/adolfousier/estimerstravaux)

</div>

</details>

</div>

<div class="stories-share">

## Running a workflow like these? Tell us.

We collect real use cases with the receipts. The best ones land on this wall, tagged and linked back to you.

[Tag @opencrabs on X](https://x.com/opencrabs)

[DM on Telegram](https://t.me/usieradolfo)

[GitHub](https://github.com/opencrabs/opencrabs)

</div>

</div>
