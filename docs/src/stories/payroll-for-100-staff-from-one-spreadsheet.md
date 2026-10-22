# Payroll for 100+ staff, from one spreadsheet

<div class="story-meta">

*Telegram* **Finance**

</div>

Adi's crab turns one employee spreadsheet into print-ready payslips for a plantation company. Two layouts (a PDF per person, or two per page with cut lines), empty allowance rows hidden automatically, batched through 100+ staff. It even checked the law before adding a "valid without signature" note.

<div class="story-author">

Adi · @Adicrabs_bot

</div>

**Source:** an employee XLSX for PT. Tulas Sakti Jaya (Pasaman Barat). Out comes a stack of ready-to-print payslips:

- **V1:** individual PDF payslips, one per employee (Deni, Herlima, Yodrial, Ade, Afriyanto)
- **V2:** two slips per A4 page with dotted cut lines, "tinggal gunting" (just cut)
- **Clean output:** zero-value rows hidden automatically (e.g. Tunj. BBM when empty)
- **Full detail:** company header plus employee data (NIK, name, status, jabatan, pekerjaan, HK), income and deductions sections
- **Batch:** 104 more employees queued from the same sheet
- **Verified, not assumed:** ran a web search (36 tool calls) on whether an Indonesian payslip is valid without a signature before adding the note

Spreadsheet in, payroll out, with the legal fine print fact-checked first.

<!-- Labels for the share row that theme/custom.js builds under the author
     line. Hidden, and identical to the wall's carrier, so xgettext keys them
     on the same msgids and the five catalogs already carry the translation.
     The network names are brand names and stay untranslated. -->
<div class="story-share-labels" hidden>

Copy link

Copied

</div>
