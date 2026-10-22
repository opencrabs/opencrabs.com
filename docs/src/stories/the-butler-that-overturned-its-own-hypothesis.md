# The butler that overturned its own hypothesis

<div class="story-meta">

*GitHub · Family* **Research**

</div>

"Check whether tomorrow's Azimuth flight was rescheduled." Someone's flying tomorrow, so the family-profile butler didn't trust one source: it scraped the airline's own ops feed from the schedule page, reverse-engineered the airport's live departure board API by reading the page's JavaScript, formed a codeshare hypothesis, then disproved it with actual departure history. Honest verdict: the flight was dropped from that day's schedule, and the airline never published a reason.

<div class="story-author">

@leshchenko1979 · family profile

</div>

> Check the Tbilisi airport website: tomorrow's Azimuth flight to Moscow Vnukovo, around 14:00, has it been rescheduled?

A one-line question with real consequences. The butler treated it like an ops incident:

- **Airline ops feed, scraped:** the schedule page embeds live ops data as JSON inside the HTML; the agent extracted and parsed it (186 flights, 4-day window, planned/actual times and statuses)
- **Airport live board, reverse-engineered:** the departure board loads from an undocumented AJAX endpoint; the agent read the page's own JS to find the correct parameter and got structured JSON back (scheduled/estimated/actual times, remarks, gates)
- **A hypothesis, then the kill shot:** the flight was missing from tomorrow's board while a partner-coded flight held the same slot, codeshare, right? Flight history said no: both numbers departed the same day ~20 minutes apart, two separate physical aircraft
- **Rotation check:** the inbound positioning leg for tomorrow was missing from the ops feed too, closing the loop
- **The honest verdict:** not delayed, not rescheduled, dropped from that date entirely. Alternatives named, and "I don't know the reason, the airline hasn't published one" said plainly instead of guessed

The same profile, the same day: dose-frequency health questions answered against peer-reviewed sources with proper caveats and red-flag lists, a forwarded astronomy fun-fact corrected with dry humor (the distance ratio is ~389×, not 400×, close enough for folklore, not for a butler), and a contested science question answered citing studies on both sides.

[Full write-up (issue #27) ↗](https://github.com/adolfousier/opencrabs.com/issues/27)

<!-- Labels for the share row that theme/custom.js builds under the author
     line. Hidden, and identical to the wall's carrier, so xgettext keys them
     on the same msgids and the five catalogs already carry the translation.
     The network names are brand names and stay untranslated. -->
<div class="story-share-labels" hidden>

Copy link

Copied

</div>
