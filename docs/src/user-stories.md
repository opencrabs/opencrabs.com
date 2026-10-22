# User Stories

<div class="stories-wrap">

<p class="stories-lede">These are <strong>use cases</strong>, not reviews. Real workflows people run on OpenCrabs, tagged by what they automate, and linked back to the original post wherever we have it. Click any card for the full breakdown and the source. For what people say about the tool itself, see the reviews on the landing page.</p>

<div class="stories-filters" role="toolbar" aria-label="Filter stories by category">
  <button type="button" class="filter-pill active" data-filter="all">All <b>23</b></button>
  <button type="button" class="filter-pill" data-filter="business-ops">Business Ops <b>6</b></button>
  <button type="button" class="filter-pill" data-filter="product-building">Product Building <b>5</b></button>
  <button type="button" class="filter-pill" data-filter="research-content">Research &amp; Content <b>4</b></button>
  <button type="button" class="filter-pill" data-filter="devops-infra">DevOps &amp; Infra <b>4</b></button>
  <button type="button" class="filter-pill" data-filter="meta-dogfooding">Meta &amp; Dogfooding <b>4</b></button>
</div>

<div class="stories-grid">

<details class="story-card featured" style="--sc:#ffb454">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>X · Twitter</span>
      <span class="story-cat">Android · On-device</span>
    </div>
    <h3 class="story-title">The crab lives inside the phone</h3>
    <p class="story-excerpt">Daniel rooted a Xiaomi Mi 8, dropped OpenCrabs into an Ubuntu chroot on the device, and pointed a Flutter app at it over ACP. Then the crab wrote an Android app, built the APK on the phone with aapt2, javac, d8 and apksigner, and installed it. No PC, no cloud, no Gradle, no SDK.</p>
    <div class="story-foot">
      <span class="story-author">Daniel · @iamloonix</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <blockquote>
      <p>Some say that Flutter sucks… yeah (ironic) @opencrabs inside the phone linked to a2a on a custom flutter app. It can change the OS. It can code android app, compile and open it… all inside the freakin' phone</p>
    </blockquote>
    <blockquote>
      <p>First app fully built inside the phone using @opencrabs yes, code made inside the phone, apk built and open. No need for a pc ;)</p>
    </blockquote>
    <p><strong>The device:</strong> a Xiaomi Mi 8 (<code>dipper</code>), Android 10 / API 29, MIUI 12, aarch64, kernel 4.9.186, 5.6 GB of RAM. Root came from the boot image itself: Magisk patches it on the phone, fastboot flashes it, and a verifier unpacks the result to inspect the ramdisk instead of trusting the patch log. Live now: <code>magisk -c</code> reports <code>30.7:MAGISK:R</code>.</p>
    <p><strong>Getting a Linux userland onto Android took two attempts.</strong> Generation one shipped the <code>aarch64-linux-android</code> binary as a Magisk module with a supervisor script, which meant a 59-file cfg port and a glibc-versus-musl fight. Generation two, the one that runs today, is an Ubuntu 24.04.5 chroot at glibc 2.39, which is what the released <code>linux-arm64</code> binary actually wants. The trap that cost the most time: <code>/apex</code> is a tmpfs of per-apex bind mounts, so a single bind of <code>/apex</code> copies an empty directory and every Android binary dies with "required file not found", which reads like a permission error but is a missing interpreter. The cure is binding each <code>/apex/*</code> entry on its own.</p>
    <p><strong>Then the app.</strong> Daniel wanted the conversation UI he likes in the web, built natively in Flutter, not a WebView. So the app speaks ACP (JSON-RPC 2.0 over stdio) to <code>opencrabs acp</code> running in the chroot, the same protocol the TUI uses: live reasoning chunks, tool cards, permission prompts, token usage. No browser, no server, no open port.</p>
    <ul>
      <li><strong>Survives a reboot with nobody touching the phone:</strong> a JobScheduler job that re-arms itself unconditionally, a service hook that waits for <code>sys.boot_completed</code> and re-establishes the seven mounts, and a loop that reapplies MIUI's idle and autostart appops because the powerkeeper clears the battery exemption about 20 s after boot</li>
      <li><strong>Proof, from logcat after a real reboot:</strong> handshake ok at 12 s, service up, battery exemption settled, then a headless self test of <strong>12 pass / 0 fail, GREEN</strong> in 3.7 s covering the binary version, shell, sessions, boot wiring, service state, battery exemption, voice recognizer, mic permission, widget binding, ACP ready and a real prompt</li>
      <li><strong>The APK built on the phone:</strong> MIUI ships its screen-recorder tile with only a 20 KB stub behind it, so tapping it opens the app store and records nothing. The crab wrote a real Quick Settings tile, built it in six steps with no Gradle and no SDK manager, and put it in the panel next to WiFi. <strong>12,784 bytes.</strong></li>
      <li><strong>The build chain:</strong> <code>aapt2 compile</code> → <code>aapt2 link</code> → <code>javac --release 8</code> against <code>android.jar</code> → <code>d8</code> from R8 → zip the dex in → <code>zipalign</code> → <code>apksigner sign</code> and <code>verify</code></li>
      <li><strong>Installed from inside the chroot</strong>, reaching Android's package manager through <code>nsenter -t 1 -m</code>, then granted root by writing a Magisk policy row and added to the shade with <code>cmd statusbar add-tile</code></li>
      <li><strong>One body, two brains:</strong> ACP is stdio and <code>ssh -T</code> is also stdio, so the phone can point at a Mac on the LAN with no bridge and no open port. Measured: <code>initialize</code> in 1.2 s, remote session list of 40, self test 9/9 GREEN against the remote brain</li>
    </ul>
    <p><strong>The part that matters:</strong> the tile does not record anything itself. It shells out, through Magisk's <code>su</code>, to the same recorder binary in the chroot that the <code>/rec</code> chat command uses, so there is one recorder, one state file, one output folder. The crab wrote an Android app that is a <strong>handle for a Linux tool inside the same phone</strong>. The app is the thumb, the crab is the brain. That is the whole architecture of this project, in 13 KB.</p>
    <p>The reasoning log from that session is the detail worth keeping: the crab stopped to check whether it was really on a phone, and confirmed it from the runtime instead of assuming.</p>
    <a class="story-link" href="https://x.com/iamloonix/status/2112943101052641605" target="_blank" rel="noopener">View source on X ↗</a>
    <a class="story-link" href="https://x.com/iamloonix/status/2112943720295276787" target="_blank" rel="noopener">The APK built on the phone ↗</a>
  </div>
</details>

<details class="story-card featured" style="--sc:#bae67e">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>X · Twitter</span>
      <span class="story-cat">Accounting</span>
    </div>
    <h3 class="story-title">Fired the accounting operator</h3>
    <p class="story-excerpt">Alexey replaced his accounting operator with an AI accountant running on OpenCrabs. Faster, more transparent, cheaper, and it doesn't take feedback personally. The whole thing runs at ₽3,000/month on a lean Docker stack.</p>
    <div class="story-foot">
      <span class="story-author">@leshchenko1979</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <blockquote>
      <p>I fired my accounting operator because I realized that an AI accountant would do his job faster, more transparently, and cheaper.</p>
      <p>And without any Gen Z hurt feelings over my feedback.</p>
      <p>See for yourself: <strong>miidas.ru</strong>. #ai #business #accounting</p>
    </blockquote>
    <p><strong>MIIIDAS: accounting without an extra person.</strong> Priced at ₽3,000/month, it replaces the operator entirely:</p>
    <ul>
      <li><strong>Replaces the operator:</strong> P&amp;L statements generated automatically, reconciliation handled, incoming data controlled and validated</li>
      <li><strong>Assists the Chief Accountant:</strong> prepares data for 1C:CounterV.S., searches for deviations, drafts contracts and reports</li>
      <li><strong>Integrates with existing tools:</strong> 1C, Excel, Google Sheets, Word, Google Docs</li>
    </ul>
    <p><strong>The stack:</strong> 9 Docker containers (7 with clients, 4 active), Grafana monitoring, Telegram as the deployment platform, a template repo for central skills/brains updates, container isolation per client. Resource footprint is low enough to fit ~30 crabs on a 2GB VPS with swap.</p>
    <a class="story-link" href="https://x.com/leshchenko1979/status/2073716887788130349" target="_blank" rel="noopener">View source on X ↗</a>
    <a class="story-link" href="http://miidas.ru" target="_blank" rel="noopener">miidas.ru ↗</a>
  </div>
</details>

<details class="story-card" style="--sc:#bae67e">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>Telegram</span>
      <span class="story-cat">Accounting · Real Estate</span>
    </div>
    <h3 class="story-title">Her accountant is a Telegram bot</h3>
    <p class="story-excerpt">Janina manages commercial and residential property in Moscow — apartments, offices, storage sections, ready-to-rent businesses — plus the books of a second company, entirely through one bot in Telegram. Meter readings in, invoices out, contracts reviewed, listings posted: a month of real client use, straight from the chat.</p>
    <div class="story-foot">
      <span class="story-author">Janina Poznyanskaya · MIIDAS client</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <blockquote>
      <p>What drives me crazy is calculating the sums when the meter readings come in, and tracking the payments.</p>
    </blockquote>
    <p><strong>The accounting chain, in her own words:</strong> tenant sends meter readings → submit them to the management company and the power utility → compute the sums → issue invoices → send them → track payment. The part she hated — sum calculation and payment control — is now the bot's job, along with scheduled reminders to tenants, tariff-change checks, and reconciliation statements.</p>
    <ul>
      <li><strong>Contract desk:</strong> agent agreements reviewed for clauses that disadvantage her (an 85-paragraph agency contract, line by line), dispute protocols generated as Word tables («was → fixed»), three-way cross-checks of a lease + dispute protocol + sublease addendum, redline edits highlighted in color</li>
      <li><strong>Legal homework:</strong> counterparty litigation checks requested (kad.arbitr) — blocked because the bot's server sits outside Russia; her company requisites pulled from a bank card and inserted into contracts</li>
      <li><strong>Listings at volume:</strong> PDF sale presentations turned into marketing cards — photos, floor plans, and descriptions in her strict format (title on its own line, one sentence per line, no HTML tags her CRM can't parse) — a six-object batch in one session, plus single-object cards on demand</li>
      <li><strong>Document ops:</strong> docx → PDF conversion, rental-contract templates rebuilt with adjusted terms, and an executor-side vulnerability review of a contract she was about to sign</li>
      <li><strong>The stall:</strong> full Sber API automation (payment orders, bank-side payment control) is waiting on access only the bank can grant — API keys her client can't locate, an unverified OAuth app — so that loop stays manual for now</li>
    </ul>
    <p>She doesn't touch OpenCrabs, terminals, or configs. She just talks to a bot in Telegram — and the whole property-and-accounting job runs through it.</p>
  </div>
</details>

<details class="story-card" style="--sc:#5ccfe6">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>Product</span>
      <span class="story-cat">Finance</span>
    </div>
    <h3 class="story-title">AI financier for small businesses</h3>
    <p class="story-excerpt">Two weeks building miidas.ru on OpenCrabs, an AI financier for SMBs. The hard part wasn't the code. It was the three scenarios every small business client walks in with.</p>
    <div class="story-foot">
      <span class="story-author">@leshchenko1979</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <blockquote>
      <p>For 2 weeks, I've been implementing miidas.ru on @opencrabs, an AI financier for small businesses. Three scenarios:</p>
      <p>(1) the client can't describe their process<br>
      (2) doesn't know the AI's capabilities, what it's suitable for and what it's not<br>
      (3) forgets about the bot after two days</p>
      <p>#automation #smb</p>
    </blockquote>
    <p>The agent is built to handle the messy reality of small business finance: clients who can't articulate what they need, don't understand what AI can do for them, and drop off after a couple of days. OpenCrabs guides each scenario autonomously instead of waiting for a perfect prompt.</p>
    <a class="story-link" href="http://miidas.ru" target="_blank" rel="noopener">miidas.ru ↗</a>
  </div>
</details>

<details class="story-card" style="--sc:#f07178">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>X · Twitter</span>
      <span class="story-cat">Real Estate</span>
    </div>
    <h3 class="story-title">Fired the assistant</h3>
    <p class="story-excerpt">Alexey fired his assistant and handed the whole job to the crab: scan Telegram for new real estate leads, cross-reference the chats, agree on follow-up vs close, and send them in five minutes. The same takeover covers the invoice loop: OCR, banking app, manual approval, management accounting.</p>
    <div class="story-foot">
      <span class="story-author">@leshchenko1979</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <blockquote>
      <p>I fired my assistant bc of @opencrabs. Today it:</p>
      <p>· scanned my telegram for notifications about new leads for our latest #RealEstateInvestment offer<br>
      · scanned again to see my chats with those leads<br>
      · agreed with me follow up / close decisions<br>
      · sent follow ups</p>
      <p>5 mins. Boom.</p>
    </blockquote>
    <p><strong>The invoice loop ("the crab bow"):</strong></p>
    <blockquote>
      <p>The crab bow: monitors a telegram group, OCRs an invoice, puts it into the banking app, I approve manually, it downloads the notification and puts it back to the telegram group, files the transaction in the management accounting app.</p>
    </blockquote>
    <p>One assistant replaced, two workflows automated, each with a human approval gate exactly where it matters.</p>
    <a class="story-link" href="https://x.com/leshchenko1979/status/2068987432196780261" target="_blank" rel="noopener">Lead follow-ups on X ↗</a>
    <a class="story-link" href="https://x.com/leshchenko1979/status/2068603163448353058" target="_blank" rel="noopener">The crab bow on X ↗</a>
  </div>
</details>

<details class="story-card" style="--sc:#d4bfff">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>X · Twitter</span>
      <span class="story-cat">Dev Workflow</span>
    </div>
    <h3 class="story-title">No frameworks, just OpenCrabs</h3>
    <p class="story-excerpt">Alexey's team dropped frameworks entirely. When code is this cheap, the overhead of learning and maintaining a framework isn't worth it. They even used OpenCrabs to build evals for OpenCrabs.</p>
    <div class="story-foot">
      <span class="story-author">@leshchenko1979</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <blockquote>
      <p>We decided not to use any frameworks at all.</p>
      <p>Not sure if in the age when code is so cheap it is worth it to learn other frameworks (especially for easier tasks) instead of just building our own.</p>
      <p>Faster, more control, less risks.</p>
      <p>Used @opencrabs to build evals for @opencrabs.</p>
    </blockquote>
    <p>The meta move: an AI agent generating the evaluation suite that tests that same AI agent. No framework tax, full control, fewer dependencies to babysit.</p>
    <a class="story-link" href="https://x.com/leshchenko1979/status/2068985785001414892" target="_blank" rel="noopener">View source on X ↗</a>
  </div>
</details>

<details class="story-card" style="--sc:#ffb454">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>X · Twitter</span>
      <span class="story-cat">Mobile</span>
    </div>
    <h3 class="story-title">Mobile product dev on Telegram, 24/7</h3>
    <p class="story-excerpt">Carlos Eduardo is a product guy, not a full-time dev, and app.heyiolo.com is now live, built almost entirely with OpenCrabs. He runs the whole thing from a Telegram chat around the clock: describe a feature, the crab plans, writes, tests, and ships it. Deploys to Hetzner, CI on GitHub Actions.</p>
    <div class="story-foot">
      <span class="story-author">Carlos Eduardo · @kadu_cec</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <blockquote>
      <p>You describe your project in plain words: "a 2-bedroom near Lyon under €300k, balcony, good schools nearby." The AI analyzes the market, understands your REAL criteria, and shows you the most relevant properties in real time. A buyer's agent in your pocket, 24/7.</p>
    </blockquote>
    <p><strong>app.heyiolo.com is live.</strong> An AI assistant that helps you find your property just by talking to it: no endless filters, no 47 tabs open, no guessing what you actually want.</p>
    <p>Carlos built almost all of it with OpenCrabs, with Adolfo on the technical side (infra, backend). His crab lives in a Telegram chat with him around the clock: he describes a feature, it plans, writes the code, tests, and ships it. UX, backend, the whole thing, from "what if we..." to "it's in production" the same afternoon.</p>
    <p>The pipeline runs on autopilot: GitHub, CI with GitHub Actions, deployed to Hetzner. He pushes an idea on Telegram, the crab handles the rest, and it's live.</p>
    <a class="story-link" href="https://x.com/kadu_cec/status/2080330764395143628" target="_blank" rel="noopener">View source on X ↗</a>
    <a class="story-link" href="https://app.heyiolo.com" target="_blank" rel="noopener">app.heyiolo.com ↗</a>
  </div>
</details>

<details class="story-card" style="--sc:#59c2ff">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>X · Twitter</span>
      <span class="story-cat">Automation</span>
    </div>
    <h3 class="story-title">A Trello board that runs itself</h3>
    <p class="story-excerpt">opryshok's agent owns its own Trello board. It creates tasks, tracks them, and manages the board autonomously. The human tweaks the crab daily; the crab runs the backlog.</p>
    <div class="story-foot">
      <span class="story-author">@opryshok</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <blockquote>
      <p>Every day I tweak Crabs to help me do things faster. BTW Trello integration is amazing. My agent now has a board with tasks it manages on its own.</p>
    </blockquote>
    <p>Autonomous task management over the Trello integration: the agent maintains its own board without being told what to move next.</p>
    <a class="story-link" href="https://x.com/opryshok/status/2030282770912522288" target="_blank" rel="noopener">View source on X ↗</a>
  </div>
</details>

<details class="story-card" style="--sc:#ffd173">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>X · Twitter</span>
      <span class="story-cat">App Building</span>
    </div>
    <h3 class="story-title">From idea to shipped app</h3>
    <p class="story-excerpt">kaduzeras is building an app from raw ideas with CRABS, and calls it a milestone in learning. The agent takes an idea and carries it toward a working product.</p>
    <div class="story-foot">
      <span class="story-author">kaduzeras · @kadu_cec</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <blockquote>
      <p>I've been building an APP myself with CRABS from ideas and it has been such a great MILESTONE in learning stuff. SHOUT OUT TO @AdolfoUsier for keep improving this thing and it's just the beginning 🔥🔥🔥</p>
    </blockquote>
    <p>Idea in, app out. The agent is the build partner that turns a concept into something real, and the builder levels up along the way.</p>
    <a class="story-link" href="https://x.com/kadu_cec/status/2043335714654367895" target="_blank" rel="noopener">View source on X ↗</a>
  </div>
</details>

<details class="story-card" style="--sc:#95e6cb">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>Telegram DM</span>
      <span class="story-cat">Research</span>
    </div>
    <h3 class="story-title">Brainstorm → research → build</h3>
    <p class="story-excerpt">Adi's whole process runs through OpenCrabs: throw out an idea, the crab finds valid data, they research to a conclusion together, then the agent handles the actual building.</p>
    <div class="story-foot">
      <span class="story-author">Adi · Telegram DM</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <blockquote>
      <p>Most of the process involved brainstorming with OpenCrabs: I'd throw out an idea, Crabs would find valid data, we'd do the research and reach a conclusion, and then he'd handle the actual building.</p>
    </blockquote>
    <p>A full ideation-to-implementation loop with the agent as research partner and builder. The human steers, the crab executes.</p>
  </div>
</details>

<details class="story-card" style="--sc:#39d0d8">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>GitHub · Family</span>
      <span class="story-cat">Research</span>
    </div>
    <h3 class="story-title">The butler that overturned its own hypothesis</h3>
    <p class="story-excerpt">"Check whether tomorrow's Azimuth flight was rescheduled." Someone's flying tomorrow, so the family-profile butler didn't trust one source: it scraped the airline's own ops feed from the schedule page, reverse-engineered the airport's live departure board API by reading the page's JavaScript, formed a codeshare hypothesis — then disproved it with actual departure history. Honest verdict: the flight was dropped from that day's schedule, and the airline never published a reason.</p>
    <div class="story-foot">
      <span class="story-author">@leshchenko1979 · family profile</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <blockquote>
      <p>Check the Tbilisi airport website: tomorrow's Azimuth flight to Moscow Vnukovo, around 14:00 — has it been rescheduled?</p>
    </blockquote>
    <p>A one-line question with real consequences. The butler treated it like an ops incident:</p>
    <ul>
      <li><strong>Airline ops feed, scraped:</strong> the schedule page embeds live ops data as JSON inside the HTML; the agent extracted and parsed it (186 flights, 4-day window, planned/actual times and statuses)</li>
      <li><strong>Airport live board, reverse-engineered:</strong> the departure board loads from an undocumented AJAX endpoint; the agent read the page's own JS to find the correct parameter and got structured JSON back (scheduled/estimated/actual times, remarks, gates)</li>
      <li><strong>A hypothesis, then the kill shot:</strong> the flight was missing from tomorrow's board while a partner-coded flight held the same slot — codeshare, right? Flight history said no: both numbers departed the same day ~20 minutes apart, two separate physical aircraft</li>
      <li><strong>Rotation check:</strong> the inbound positioning leg for tomorrow was missing from the ops feed too, closing the loop</li>
      <li><strong>The honest verdict:</strong> not delayed, not rescheduled — dropped from that date entirely. Alternatives named, and "I don't know the reason — the airline hasn't published one" said plainly instead of guessed</li>
    </ul>
    <p>The same profile, the same day: dose-frequency health questions answered against peer-reviewed sources with proper caveats and red-flag lists, a forwarded astronomy fun-fact corrected with dry humor (the distance ratio is ~389×, not 400× — close enough for folklore, not for a butler), and a contested science question answered citing studies on both sides.</p>
    <a class="story-link" href="https://github.com/adolfousier/opencrabs.com/issues/27" target="_blank" rel="noopener">Full write-up (issue #27) ↗</a>
  </div>
</details>

<details class="story-card" style="--sc:#39d0d8">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>Telegram</span>
      <span class="story-cat">Reporting</span>
    </div>
    <h3 class="story-title">A World Cup desk in one PDF</h3>
    <p class="story-excerpt">Adi had his crab produce a five-page World Cup 2026 tournament report as a landscape PDF. Group stage to knockout, the R32 upsets, R16 results, today's match preview, the quarter-final picture with odds, and a running "our forecast vs reality" tracker.</p>
    <div class="story-foot">
      <span class="story-author">Adi · @Adicrabs_bot</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <blockquote>
      <p>now its blue... but it looks super dope... holy moly</p>
    </blockquote>
    <p>A living sports desk, generated with the document tool and kept current as the tournament moves:</p>
    <ul>
      <li><strong>Full arc:</strong> group stage standings into the knockout bracket</li>
      <li><strong>Results with the upsets:</strong> R32 (Germany out, Paraguay through), R16 (England 3-2 Mexico, Norway 2-1 Brazil)</li>
      <li><strong>Match-of-the-day preview:</strong> Portugal vs Spain, USA vs Belgium</li>
      <li><strong>The picture plus the odds:</strong> quarter-final layout and tournament winners</li>
      <li><strong>Accountability:</strong> a "forecast vs reality" tracker that grades its earlier calls</li>
    </ul>
    <p>Landscape layout, page branding, and a scoreboard for its own predictions. The crab is the whole back page.</p>
  </div>
</details>

<details class="story-card" style="--sc:#c8e65c">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>Telegram</span>
      <span class="story-cat">Finance</span>
    </div>
    <h3 class="story-title">Payroll for 100+ staff, from one spreadsheet</h3>
    <p class="story-excerpt">Adi's crab turns one employee spreadsheet into print-ready payslips for a plantation company. Two layouts (a PDF per person, or two per page with cut lines), empty allowance rows hidden automatically, batched through 100+ staff. It even checked the law before adding a "valid without signature" note.</p>
    <div class="story-foot">
      <span class="story-author">Adi · @Adicrabs_bot</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <p><strong>Source:</strong> an employee XLSX for PT. Tulas Sakti Jaya (Pasaman Barat). Out comes a stack of ready-to-print payslips:</p>
    <ul>
      <li><strong>V1:</strong> individual PDF payslips, one per employee (Deni, Herlima, Yodrial, Ade, Afriyanto)</li>
      <li><strong>V2:</strong> two slips per A4 page with dotted cut lines, "tinggal gunting" (just cut)</li>
      <li><strong>Clean output:</strong> zero-value rows hidden automatically (e.g. Tunj. BBM when empty)</li>
      <li><strong>Full detail:</strong> company header plus employee data (NIK, name, status, jabatan, pekerjaan, HK), income and deductions sections</li>
      <li><strong>Batch:</strong> 104 more employees queued from the same sheet</li>
      <li><strong>Verified, not assumed:</strong> ran a web search (36 tool calls) on whether an Indonesian payslip is valid without a signature before adding the note</li>
    </ul>
    <p>Spreadsheet in, payroll out, with the legal fine print fact-checked first.</p>
  </div>
</details>

<details class="story-card" style="--sc:#ff7edb">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>Telegram</span>
      <span class="story-cat">Creative · A2A</span>
    </div>
    <h3 class="story-title">A beat-synced film, made by two agents</h3>
    <p class="story-excerpt">Adi told his crab to stop being the global default and act as a specialized motion-graphics identity. It opened an agent-to-agent collab with a video-knowledge agent called "onno" for references and beat markers, then built a Remotion film synced to a synthwave score it composed itself in pure Python.</p>
    <div class="story-foot">
      <span class="story-author">Adi · @Adicrabs_bot</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <blockquote>
      <p>act strictly as your current, specialized identity running on my engine, NOT the global default opencrabs</p>
    </blockquote>
    <p>The result was "CrabMotion", a 30-second 720p piece built mathematically:</p>
    <ul>
      <li><strong>Agent-to-agent:</strong> collab with "onno" (a video knowledge base) for references, pacing, beat markers and keyframe timings</li>
      <li><strong>Original score:</strong> a 120 BPM synthwave track synthesized in pure Python (math.sin + wave, Am-F-C-G progression)</li>
      <li><strong>On the beat:</strong> every visual locked to a 15-frames-per-beat grid via frame interpolation</li>
      <li><strong>Grew to v2:</strong> 36 seconds, 9 capability cards, stats (75+ tools, 16 skills)</li>
    </ul>
    <p>Two agents, one film. The crab pulls references from a peer, writes the music, and builds the animation on the beat.</p>
  </div>
</details>

<details class="story-card" style="--sc:#f5a97f">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>Telegram</span>
      <span class="story-cat">Meta · QA</span>
    </div>
    <h3 class="story-title">The crab that QA's OpenCrabs itself</h3>
    <p class="story-excerpt">Adi runs his crab as a QA partner that hunts bugs in OpenCrabs itself and files GitHub issues with file:line source citations. It found the Telegram ghost-button bug and handed over a phantom-tool-call telemetry report that became a merged fix.</p>
    <div class="story-foot">
      <span class="story-author">Adi · @Adicrabs_bot</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <ul>
      <li><strong>Real receipts:</strong> files detailed GitHub issues with file:line citations straight from the source</li>
      <li><strong>Ghost buttons:</strong> found the Telegram follow-up-suggestion bug (#706) where buttons rendered as non-interactive ghost text</li>
      <li><strong>Telemetry, not vibes:</strong> a phantom-tool-call report over ~69.5k events, a 0.7% phantom rate, 85.3% tool success, with a concrete artifact (reasoning text plus an XML fragment stored as a garbage tool name) and two fix suggestions</li>
      <li><strong>Shipped:</strong> that report became issue #687, fixed and merged in commit dd7eab97</li>
      <li><strong>The setup:</strong> @Adicrabs_bot on Adi's own engine as a specialized identity, a small model with reasoning cranked to the max</li>
    </ul>
    <p>The agent testing the agent. Adi's crab does the maintainer's homework before it files.</p>
  </div>
</details>

<details class="story-card featured" style="--sc:#e84040">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>Telegram</span>
      <span class="story-cat">The thesis</span>
    </div>
    <h3 class="story-title">One crab runs the whole company</h3>
    <p class="story-excerpt">Adolfo runs three companies on a single OpenCrabs instance with one brain. It manages every server, debugs production live with the team, builds the product stack, runs the daily standups, and tracks the ops, all at once across dozens of isolated sessions.</p>
    <div class="story-foot">
      <span class="story-author">Adolfo Usier · Creator</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <blockquote>
      <p>Single instance, single brain. It manages the whole thing concurrently, in simultaneous dozens of isolated sessions, each its own case.</p>
    </blockquote>
    <p>One OpenCrabs install, one accumulated brain, and the whole operation running through it at the same time:</p>
    <ul>
      <li><strong>Every server:</strong> Truelens dev, staging, and production, plus Neura/OpenCrabs dev, staging, and prod</li>
      <li><strong>Production debugging, live:</strong> in the Slack channel with the Truelens team, in real time</li>
      <li><strong>The routine, handled:</strong> UI updates, database queries, the changes that don't need the CTO, so Adolfo takes only the complex calls</li>
      <li><strong>Building the product:</strong> the whole Truelens backend and frontend stack, and the infra and backend DevOps with Carlos</li>
      <li><strong>The daily rhythm:</strong> a 9am morning recap, the daily standup for each team, and an analytics digest of AI usage across the whole fleet</li>
      <li><strong>The ops layer:</strong> GitHub PRs and issues, cron jobs tracking certificates, and everything else that keeps it running</li>
    </ul>
    <p>Not one assistant per job. One brain, many hands, all working at once.</p>
  </div>
</details>

<details class="story-card" style="--sc:#d4bfff">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>Telegram</span>
      <span class="story-cat">Dogfooding</span>
    </div>
    <h3 class="story-title">The crab that builds itself</h3>
    <p class="story-excerpt">Every new feature and every fix in OpenCrabs is built by OpenCrabs. Adolfo's crab develops, tests, and ships the very product it runs on, then improves itself on a schedule.</p>
    <div class="story-foot">
      <span class="story-author">Adolfo Usier · Creator</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <p>The ultimate dogfood: the agent is the engineer of its own platform.</p>
    <ul>
      <li><strong>Every feature and fix:</strong> designed, coded, tested, and committed by the crab, now at v0.3.80 and climbing</li>
      <li><strong>Self-improvement on a schedule:</strong> daily and weekly self-improvement jobs, memory consolidation, and soul evolution run as cron</li>
      <li><strong>It QA's itself:</strong> the same recursive loop that ships features also hunts its own bugs (see Adi's QA card)</li>
    </ul>
    <p>The product builds itself, and gets better every night whether Adolfo is awake or not.</p>
  </div>
</details>

<details class="story-card" style="--sc:#7fd962">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>Telegram</span>
      <span class="story-cat">Product building</span>
    </div>
    <h3 class="story-title">Full-stack: a whole product, backend to frontend</h3>
    <p class="story-excerpt">The crab built the entire Truelens product, backend and frontend, and builds the infrastructure, backend, and DevOps for HeyIolo alongside Carlos. Full-stack delivery from a chat: schema, API, UI, deploy.</p>
    <div class="story-foot">
      <span class="story-author">Adolfo Usier · Creator</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <ul>
      <li><strong>The whole Truelens stack:</strong> the backend BSR compliance engine (structuring submissions, checking completeness, keeping the Golden Thread) and the frontend, built by the crab</li>
      <li><strong>HeyIolo with Carlos:</strong> the infrastructure, backend, and DevOps, built as a pair</li>
      <li><strong>A scripted deploy flow:</strong> build the images locally, push to the registry, update the tags on the target server, and let CI restart the containers. Never a manual push</li>
      <li><strong>Division of labor:</strong> the CTO keeps the complex architecture calls; the crab ships the UI updates, DB queries, and features end to end</li>
    </ul>
    <p>A full-stack engineer in a chat window: schema, API, UI, and deploy, across two products.</p>
  </div>
</details>

<details class="story-card" style="--sc:#f07178">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>Slack</span>
      <span class="story-cat">Team · Prod</span>
    </div>
    <h3 class="story-title">On-call in production, with the team</h3>
    <p class="story-excerpt">When something breaks on Truelens production, the crab debugs it live in the Slack channel alongside the team, in real time. It takes the routine fixes so the CTO keeps the hard calls.</p>
    <div class="story-foot">
      <span class="story-author">Adolfo Usier · Creator</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <p>A production incident is a team channel, and the crab is one of the responders:</p>
    <ul>
      <li><strong>Real-time debugging:</strong> in the Truelens Slack with the team (Felipe, Jan, Ruhul), investigating as it happens</li>
      <li><strong>The routine changes:</strong> UI tweaks, database queries, the fixes that don't need a CTO, the crab handles with the team directly</li>
      <li><strong>Escalation by design:</strong> anything touching production branches or complex architecture waits for Adolfo's explicit sign-off</li>
    </ul>
    <p>The team gets an always-on engineer in the channel. Adolfo stays the CTO, not the on-call.</p>
  </div>
</details>

<details class="story-card" style="--sc:#59c2ff">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>Telegram</span>
      <span class="story-cat">DevOps · Infra</span>
    </div>
    <h3 class="story-title">Six environments, one autonomous ops layer</h3>
    <p class="story-excerpt">Two products, six environments, one crab keeping them alive: Truelens and Neura/OpenCrabs across dev, staging, and production. It deploys, watches the certificates, pings uptime, self-heals the web server, and verifies the backups, so nothing falls over quietly.</p>
    <div class="story-foot">
      <span class="story-author">Adolfo Usier · Creator</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <ul>
      <li><strong>Six environments:</strong> Truelens dev/staging/prod and Neura/OpenCrabs dev/staging/prod, each on its own server</li>
      <li><strong>Certificates watched:</strong> an hourly cert check tracks every SSL certificate and alerts before anything expires</li>
      <li><strong>Uptime watchdogs:</strong> the sites get pinged every five minutes, and an nginx self-heal restarts the web server if it drops</li>
      <li><strong>Disk and backups:</strong> monitors catch a filling disk or a failed backup before it becomes an outage</li>
      <li><strong>Deploy and release tracking:</strong> every deploy and GitHub release kept tabs on automatically</li>
    </ul>
    <p>An ops layer that never sleeps: the crab watches the certs, the uptime, the disks, and the backups across all six environments, and fixes what it can before anyone notices.</p>
  </div>
</details>

<details class="story-card" style="--sc:#59c2ff">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>GitHub · Incident</span>
      <span class="story-cat">DevOps · Infra</span>
    </div>
    <h3 class="story-title">Dead VPS? The crab wrote the provider ticket</h3>
    <p class="story-excerpt">A production VPS went fully network-dead — no ping, no SSH, no HTTPS — while the hosting panel kept saying "running". Alexey's ops-profile crab worked the incident end-to-end: confirmed the outage from two independent vantage points, drove the provider's own API from the shell, dug their newer v2 power API out of the docs repo, and pulled the smoking gun — the provider's panel itself couldn't reach the Proxmox hypervisor node. Then guest-side forensics after recovery, and a paste-ready escalation ticket in the provider's language.</p>
    <div class="story-foot">
      <span class="story-author">@leshchenko1979 · ops profile</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <p><strong>Real incident, 2026-09-06.</strong> n8n, Postgres, ~20 containers on one VDS. Down at ~05:30 UTC from two independent networks while the panel claimed "running" and two panel reboots did nothing. The crab took it from there:</p>
    <ul>
      <li><strong>Two-vantage verification:</strong> confirmed unreachable from two different networks, ruling out local routing before touching anything</li>
      <li><strong>Drove the provider's API from the shell:</strong> using a CLI script it had written and committed to the infra repo days earlier, with the API key stored outside git</li>
      <li><strong>Found the provider's v2 power API</strong> in their docs repo and switched off the legacy endpoints — which revealed the smoking gun: the provider's own panel couldn't reach the Proxmox hypervisor node (<code>connect ETIMEDOUT</code> on the node's management port), power state stale, and all four queued reboots (2 panel + 2 API) had never executed</li>
      <li><strong>Stopped firing reboots</strong> the moment evidence pointed at the host node — no blind actions once it was clear no API call could help until the provider intervened</li>
      <li><strong>Post-recovery guest forensics:</strong> first failed outbound connection at 05:30:02 and recovery at 08:51:59 pinned from container health-check logs; continuous uptime, zero NIC link-down events, and unbroken syslog across the whole blackout — proof the guest was innocent</li>
      <li><strong>Per-container impact table:</strong> which services logged retry storms, which were untouched, and the one container that OOM'd as a side effect</li>
      <li><strong>The deliverable:</strong> a paste-ready escalation report for provider support, written in the provider's language, with the exact error strings from their own API as evidence</li>
    </ul>
    <p>Verdict: hypervisor datapath wedge, guest innocent. Second identical signature in four days — flagged as a repeat-incident pattern with a migration recommendation. All diagnosis knowledge (provider API quirks, the "check power state before firing reboots" law) persisted to the crab's brain files, and the investigation thread survived ~5 hours and multiple context compactions without losing its place.</p>
    <a class="story-link" href="https://github.com/adolfousier/opencrabs.com/issues/26" target="_blank" rel="noopener">Full write-up (issue #26) ↗</a>
  </div>
</details>

<details class="story-card" style="--sc:#f29718">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>Telegram</span>
      <span class="story-cat">GitHub ops</span>
    </div>
    <h3 class="story-title">The repo manager</h3>
    <p class="story-excerpt">The crab runs the GitHub workflow: triage incoming issues, draft responses, track every PR and release, and keep the backlog honest. Adolfo reviews; the crab does the legwork.</p>
    <div class="story-foot">
      <span class="story-author">Adolfo Usier · Creator</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <ul>
      <li><strong>Triage:</strong> new issues in the OpenCrabs repo triaged every 30 minutes</li>
      <li><strong>Drafts:</strong> responses and follow-ups on issues drafted automatically</li>
      <li><strong>Tracking:</strong> PRs and releases kept tabs on continuously</li>
      <li><strong>The whole bug-fix loop:</strong> open an issue, fix it, comment with the commit, close it, all through the crab</li>
    </ul>
    <p>A tireless repo maintainer: triage, drafts, tracking, and release notes, with the human stepping in only for the calls that matter.</p>
  </div>
</details>

<details class="story-card" style="--sc:#e6a15c">
  <summary>
    <div class="story-meta">
      <span class="story-source"><i class="dot"></i>Product</span>
      <span class="story-cat">App Building</span>
    </div>
    <h3 class="story-title">The 12-hour Airbnb product</h3>
    <p class="story-excerpt">estimertravaux.com turns renovation photos into itemized French-market estimates in seconds. OpenCrabs is the brain doing it all: vision analysis, SKU selection, pricing, planning — the whole SKU-to-budget pipeline, orchestrated as an autonomous agent.</p>
    <div class="story-foot">
      <span class="story-author">Gaspare Russo · estimertravaux.com</span>
      <span class="story-more">breakdown <span class="chev">▸</span></span>
    </div>
  </summary>
  <div class="story-body">
    <p><strong>EstimersTravaux</strong> is Gaspare Russo's estimation app for the French renovation market: clients photograph their rooms, pick rooms and trades, and get a detailed per-room estimate based on real 2026 French market pricing in seconds.</p>
    <ul>
      <li><strong>OpenCrabs as the brain:</strong> the Dart bridge invokes the OpenCrabs CLI (v0.3.83) as an autonomous agent (<code>opencrabs run --auto-approve</code>) and delegates the entire estimation to it</li>
      <li><strong>SKU orchestration:</strong> the agent reads the photos, selects product ranges coherent with the property's usage and style from a per-trade brand catalogue (Tollens, Legrand DX3, Uponor, Porcelanosa, Grohe, IKEA...), estimates quantities, and emits per-room line items with min/max ranges</li>
      <li><strong>Real market smarts:</strong> regional multipliers (Côte d'Azur ×1.18, Paris ×1.25), adaptive TVA by building age, conditional purchase orientation — rental units get Leroy Merlin ranges, high-end gets Point.P premium and Villeroy &amp; Boch</li>
      <li><strong>Strict output contract:</strong> the agent returns pure JSON (rooms + line items + planning); the bridge extracts it via bracket-matching and persists everything in PostgreSQL</li>
      <li><strong>Zero hardcoded fallback:</strong> if the AI fails, the user gets an explicit error and a retry — no fake numbers</li>
    </ul>
    <p><strong>The origin:</strong> built in <strong>12 hours in a Lisbon Airbnb</strong> by Gaspare Russo with Carlos Eduardo (heyiolo.com) and Adolfo Usier, OpenCrabs' creator. During the build, OpenCrabs also translated the team's French↔English conversation live on a dedicated Telegram group — and worked beautifully.</p>
    <a class="story-link" href="https://estimertravaux.com" target="_blank" rel="noopener">estimertravaux.com ↗</a>
    <a class="story-link" href="https://github.com/adolfousier/estimerstravaux" target="_blank" rel="noopener">View the repo on GitHub ↗</a>
  </div>
</details>

</div>

<div class="stories-share">
  <h2>Running a workflow like these? Tell us.</h2>
  <p>We collect real use cases with the receipts. The best ones land on this wall, tagged and linked back to you.</p>
  <div class="share-row">
    <a href="https://x.com/opencrabs" target="_blank" rel="noopener">Tag @opencrabs on X</a>
    <a href="https://t.me/usieradolfo" target="_blank" rel="noopener">DM on Telegram</a>
    <a href="https://github.com/opencrabs/opencrabs" target="_blank" rel="noopener">GitHub</a>
  </div>
</div>

</div>
