# Fired the accounting operator

<div class="story-meta">

*X · Twitter* **Accounting**

</div>

Alexey replaced his accounting operator with an AI accountant running on OpenCrabs. Faster, more transparent, cheaper, and it doesn't take feedback personally. The whole thing runs at ₽3,000/month on a lean Docker stack.

<div class="story-author">

@leshchenko1979

</div>

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
