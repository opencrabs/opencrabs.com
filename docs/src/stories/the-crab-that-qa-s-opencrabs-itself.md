# The crab that QA's OpenCrabs itself

<div class="story-meta">

*Telegram* **Meta · QA**

</div>

Adi runs his crab as a QA partner that hunts bugs in OpenCrabs itself and files GitHub issues with file:line source citations. It found the Telegram ghost-button bug and handed over a phantom-tool-call telemetry report that became a merged fix.

<div class="story-author">

Adi · @Adicrabs_bot

</div>

- **Real receipts:** files detailed GitHub issues with file:line citations straight from the source
- **Ghost buttons:** found the Telegram follow-up-suggestion bug (#706) where buttons rendered as non-interactive ghost text
- **Telemetry, not vibes:** a phantom-tool-call report over ~69.5k events, a 0.7% phantom rate, 85.3% tool success, with a concrete artifact (reasoning text plus an XML fragment stored as a garbage tool name) and two fix suggestions
- **Shipped:** that report became issue #687, fixed and merged in commit dd7eab97
- **The setup:** @Adicrabs_bot on Adi's own engine as a specialized identity, a small model with reasoning cranked to the max

The agent testing the agent. Adi's crab does the maintainer's homework before it files.
