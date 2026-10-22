/* The shared site chrome behaviour: one script for every page.
 *
 * It owns the header menu, the macOS download popover, the footer community
 * popover, the language picker and the two live counts (GitHub stars and
 * distinct callers). The markup comes from the `chrome` crate, so this file
 * only ever looks for the data hooks that markup carries, never for a page.
 *
 * Everything is delegated from `document`, which means a page that re-renders
 * its header (the landing rebuilds it on a locale change) keeps working
 * without re-attaching anything.
 */
(function () {
  "use strict";

  var REPO = "opencrabs/opencrabs";
  var CDN = "https://github.com/" + REPO + "/releases/download/";
  var LATEST = "https://github.com/" + REPO + "/releases/latest";
  var API = "https://api.github.com/repos/" + REPO + "/releases/latest";

  function num(n) {
    return Number(n).toLocaleString("en-US");
  }

  function isMac() {
    return /Mac|iPhone|iPad/i.test(navigator.userAgent) || navigator.platform === "MacIntel";
  }

  function setMenu(open) {
    var links = document.querySelector(".nav-links");
    var toggle = document.querySelector(".nav-toggle");
    if (links) links.classList.toggle("open", open);
    if (toggle) {
      toggle.classList.toggle("active", open);
      toggle.setAttribute("aria-expanded", open ? "true" : "false");
    }
  }

  function copy(text, button) {
    var was = button.textContent;
    var done = function () {
      button.textContent = "\u2713";
      setTimeout(function () { button.textContent = was; }, 1500);
    };
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(done, function () {});
    }
  }

  document.addEventListener("click", function (ev) {
    var target = ev.target;
    if (!target || !target.closest) return;

    // The Download item comes first: on macOS it opens the brew popover
    // instead of navigating, and it must not be swallowed by the rule that
    // closes the menu on any nav link.
    var download = target.closest("[data-chrome-download]");
    if (download && isMac()) {
      ev.preventDefault();
      var popover = document.querySelector("[data-chrome-dl-popover]");
      if (popover) popover.classList.toggle("open");
      return;
    }

    if (target.closest(".nav-toggle")) {
      ev.preventDefault();
      var links = document.querySelector(".nav-links");
      setMenu(!(links && links.classList.contains("open")));
      return;
    }

    if (target.closest(".nav-links a")) {
      setMenu(false);
      return;
    }

    var comm = target.closest(".footer-comm-toggle");
    if (comm) {
      ev.preventDefault();
      var wrap = comm.closest(".footer-comm-wrap");
      var panel = wrap && wrap.querySelector(".footer-comm-popover");
      if (!panel) return;
      var open = panel.classList.toggle("open");
      comm.setAttribute("aria-expanded", open ? "true" : "false");
      return;
    }

    var copyButton = target.closest("[data-chrome-copy]");
    if (copyButton) {
      ev.preventDefault();
      copy(copyButton.getAttribute("data-chrome-copy"), copyButton);
      return;
    }

    var close = target.closest("[data-chrome-dl-close]");
    if (close) {
      ev.preventDefault();
      var panelToClose = close.closest(".dl-popover");
      if (panelToClose) panelToClose.classList.remove("open");
      return;
    }

    // A click anywhere else closes an open download popover.
    var openPopover = document.querySelector(".dl-popover.open");
    if (openPopover && !openPopover.contains(target)) openPopover.classList.remove("open");

    // The menu is a panel over the page, so a click outside it dismisses it.
    if (!target.closest("nav")) setMenu(false);
  });

  // Escape closes the menu and returns focus to the hamburger.
  document.addEventListener("keydown", function (ev) {
    if (ev.key !== "Escape" && ev.key !== "Esc") return;
    var toggle = document.querySelector(".nav-toggle");
    if (!toggle || toggle.getAttribute("aria-expanded") !== "true") return;
    setMenu(false);
    toggle.focus();
  });

  // The language picker writes the choice and lets the page react. The landing
  // listens for the event and switches locale without a reload; a page with no
  // i18n context simply never renders the picker.
  document.addEventListener("change", function (ev) {
    var picker = ev.target;
    if (!picker || !picker.classList || !picker.classList.contains("lang-switcher")) return;
    try { localStorage.setItem("oc-lang", picker.value); } catch (e) {}
    window.dispatchEvent(new CustomEvent("oc-lang", { detail: picker.value }));
  });

  // GitHub stars. The markup renders the count when the page already knows it,
  // so this only fills an empty slot.
  fetch("https://api.github.com/repos/" + REPO)
    .then(function (r) { return r.ok ? r.json() : null; })
    .then(function (d) {
      if (!d || typeof d.stargazers_count !== "number" || d.stargazers_count <= 0) return;
      var n = d.stargazers_count;
      var slot = document.querySelector("[data-chrome-stars]");
      if (slot && !slot.textContent.trim()) slot.textContent = " \u2605 " + num(n);
      // The sentence comes from the page, so each locale keeps its own
      // wording; only the number is filled in here.
      var cta = document.querySelector("[data-chrome-star-cta]");
      if (cta && cta.getAttribute("data-filled") === "0") {
        var template = cta.getAttribute("data-chrome-star-count") || "";
        if (template) {
          cta.textContent = template.replace("{n}", num(n));
          cta.setAttribute("data-filled", "1");
        }
      }
    })
    .catch(function () {});

  // Distinct callers of the inference endpoint, from the same public snapshot
  // the stats page draws. A missing or stale file leaves the pill hidden
  // rather than printing a zero nobody measured.
  fetch("/stats-data/public.json")
    .then(function (r) { return r.ok ? r.json() : null; })
    .then(function (d) {
      if (!d || d.schema < 2 || !d.totals) return;
      var n = d.totals.unique_ips;
      if (typeof n !== "number" || n <= 0) return;
      var pills = document.querySelectorAll("[data-chrome-callers]");
      Array.prototype.forEach.call(pills, function (pill) {
        var slot = pill.querySelector("[data-chrome-callers-num]");
        if (slot && !slot.textContent.trim()) slot.textContent = num(n);
        pill.removeAttribute("hidden");
      });
    })
    .catch(function () {});

  // On macOS the Download item offers the brew one-liner before the binary.
  // The markup ships on every page; only the behaviour is conditional.
  (function () {
    if (!isMac() || !document.querySelector("[data-chrome-download]")) return;
    fetch(API, { headers: { Accept: "application/vnd.github+json" } })
      .then(function (r) { return r.ok ? r.json() : null; })
      .then(function (rel) {
        if (!rel || !rel.tag_name) return;
        var tag = rel.tag_name;
        var url = CDN + tag + "/opencrabs-" + tag + "-macos-arm64.tar.gz";
        var nav = document.querySelector("[data-chrome-download]");
        var direct = document.querySelector("[data-chrome-direct]");
        if (nav) nav.setAttribute("href", url);
        if (direct) direct.setAttribute("href", url);
      })
      .catch(function () {
        var direct = document.querySelector("[data-chrome-direct]");
        if (direct) direct.setAttribute("href", LATEST);
      });
  })();
})();
