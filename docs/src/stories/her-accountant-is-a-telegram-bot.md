# Her accountant is a Telegram bot

<div class="story-meta">

*Telegram* **Accounting · Real Estate**

</div>

Janina manages commercial and residential property in Moscow (apartments, offices, storage sections, ready-to-rent businesses) plus the books of a second company, entirely through one bot in Telegram. Meter readings in, invoices out, contracts reviewed, listings posted: a month of real client use, straight from the chat.

<div class="story-author">

Janina Poznyanskaya · MIIDAS client

</div>

> What drives me crazy is calculating the sums when the meter readings come in, and tracking the payments.

**The accounting chain, in her own words:** tenant sends meter readings → submit them to the management company and the power utility → compute the sums → issue invoices → send them → track payment. The part she hated (sum calculation and payment control) is now the bot's job, along with scheduled reminders to tenants, tariff-change checks, and reconciliation statements.

- **Contract desk:** agent agreements reviewed for clauses that disadvantage her (an 85-paragraph agency contract, line by line), dispute protocols generated as Word tables («was → fixed»), three-way cross-checks of a lease + dispute protocol + sublease addendum, redline edits highlighted in color
- **Legal homework:** counterparty litigation checks requested (kad.arbitr), blocked because the bot's server sits outside Russia; her company requisites pulled from a bank card and inserted into contracts
- **Listings at volume:** PDF sale presentations turned into marketing cards (photos, floor plans, and descriptions in her strict format (title on its own line, one sentence per line, no HTML tags her CRM can't parse)) a six-object batch in one session, plus single-object cards on demand
- **Document ops:** docx → PDF conversion, rental-contract templates rebuilt with adjusted terms, and an executor-side vulnerability review of a contract she was about to sign
- **The stall:** full Sber API automation (payment orders, bank-side payment control) is waiting on access only the bank can grant (API keys her client can't locate, an unverified OAuth app) so that loop stays manual for now

She doesn't touch OpenCrabs, terminals, or configs. She just talks to a bot in Telegram, and the whole property-and-accounting job runs through it.
