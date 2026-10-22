# The 12-hour Airbnb product

<div class="story-meta">

*Product* **App Building**

</div>

estimertravaux.com turns renovation photos into itemized French-market estimates in seconds. OpenCrabs is the brain doing it all: vision analysis, SKU selection, pricing, planning, the whole SKU-to-budget pipeline, orchestrated as an autonomous agent.

<div class="story-author">

Gaspare Russo · estimertravaux.com

</div>

**EstimersTravaux** is Gaspare Russo's estimation app for the French renovation market: clients photograph their rooms, pick rooms and trades, and get a detailed per-room estimate based on real 2026 French market pricing in seconds.

- **OpenCrabs as the brain:** the Dart bridge invokes the OpenCrabs CLI (v0.3.83) as an autonomous agent (`opencrabs run --auto-approve`) and delegates the entire estimation to it
- **SKU orchestration:** the agent reads the photos, selects product ranges coherent with the property's usage and style from a per-trade brand catalogue (Tollens, Legrand DX3, Uponor, Porcelanosa, Grohe, IKEA...), estimates quantities, and emits per-room line items with min/max ranges
- **Real market smarts:** regional multipliers (Côte d'Azur ×1.18, Paris ×1.25), adaptive TVA by building age, conditional purchase orientation, rental units get Leroy Merlin ranges, high-end gets Point.P premium and Villeroy & Boch
- **Strict output contract:** the agent returns pure JSON (rooms + line items + planning); the bridge extracts it via bracket-matching and persists everything in PostgreSQL
- **Zero hardcoded fallback:** if the AI fails, the user gets an explicit error and a retry, no fake numbers

**The origin:** built in **12 hours in a Lisbon Airbnb** by Gaspare Russo with Carlos Eduardo (heyiolo.com) and Adolfo Usier, OpenCrabs' creator. During the build, OpenCrabs also translated the team's French↔English conversation live on a dedicated Telegram group, and worked beautifully.

[estimertravaux.com ↗](https://estimertravaux.com)

[View the repo on GitHub ↗](https://github.com/adolfousier/estimerstravaux)

<!-- Labels for the share row that theme/custom.js builds under the author
     line. Hidden, and identical to the wall's carrier, so xgettext keys them
     on the same msgids and the five catalogs already carry the translation.
     The network names are brand names and stay untranslated. -->
<div class="story-share-labels" hidden>

Copy link

Copied

</div>
