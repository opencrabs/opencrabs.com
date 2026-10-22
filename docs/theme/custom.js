// Make the book title link back to the main site
document.addEventListener('DOMContentLoaded', function() {
    var title = document.querySelector('h1.menu-title');
    if (title) {
        title.style.cursor = 'pointer';
        title.addEventListener('click', function() {
            window.location.href = 'https://opencrabs.com';
        });
    }
});

// User Stories: filter pills, per-card share buttons, and a URL that keeps up.
//
// The pills used to only toggle a class, so a filtered wall had no address to
// copy. The filter is now ?filter=<slug>, pushed on click and read on load, so
// a link opens the same view and the back button walks the filters.
//
// Every card carries a stable id (story-<slug>, derived from the English title
// so it is the same string in all six books). That id is what the share
// buttons point at, and opening a card writes it to the address bar so a URL
// copied from there points at the story on screen.
//
// Guarded so it only runs on the user-stories page.
document.addEventListener('DOMContentLoaded', function() {
    var bar = document.querySelector('.stories-filters');
    var grid = document.querySelector('.stories-grid');
    if (!bar || !grid) return;

    // The cards carry their filter group in data-group. Matching on the visible
    // category text would stop working the moment that label is translated,
    // which it now is on every non-English page.
    var cards = Array.prototype.slice.call(grid.querySelectorAll('.story-card'));
    function cardGroup(card) {
        return card.getAttribute('data-group') || 'other';
    }

    var pills = Array.prototype.slice.call(bar.querySelectorAll('.filter-pill'));

    // The label is markdown so xgettext can see it, which leaves it wrapped in
    // a <p>. Flatten that and rebuild the count from the cards themselves, so
    // the numbers can never drift from the wall.
    pills.forEach(function(pill) {
        var label = pill.querySelector('p');
        if (label) pill.textContent = label.textContent.trim();

        var f = pill.getAttribute('data-filter');
        var n = (f === 'all')
            ? cards.length
            : cards.filter(function(c) { return cardGroup(c) === f; }).length;
        var b = pill.querySelector('b');
        if (!b) {
            b = document.createElement('b');
            pill.appendChild(b);
        }
        b.textContent = n;
    });

    var filters = pills.map(function(p) { return p.getAttribute('data-filter'); });

    function applyFilter(f, writeUrl) {
        if (filters.indexOf(f) === -1) f = 'all';
        pills.forEach(function(p) {
            p.classList.toggle('active', p.getAttribute('data-filter') === f);
        });
        cards.forEach(function(card) {
            var show = (f === 'all') || (cardGroup(card) === f);
            card.classList.toggle('is-hidden', !show);
        });
        if (writeUrl) {
            var url = new URL(window.location.href);
            if (f === 'all') url.searchParams.delete('filter');
            else url.searchParams.set('filter', f);
            window.history.pushState({ filter: f }, '', url);
        }
        return f;
    }

    pills.forEach(function(pill) {
        pill.addEventListener('click', function() {
            applyFilter(pill.getAttribute('data-filter'), true);
        });
    });

    // A shared link carries the view in the query string.
    function filterFromUrl() {
        return new URLSearchParams(window.location.search).get('filter') || 'all';
    }
    applyFilter(filterFromUrl(), false);

    window.addEventListener('popstate', function() {
        applyFilter(filterFromUrl(), false);
    });

    // ---- share buttons -------------------------------------------------
    // Labels live in the markdown as a hidden carrier so the five catalogs
    // translate them. Read them, then drop the carrier: it exists for the
    // extractor, not for the page.
    var labels = { copy: 'Copy link', copied: 'Copied' };
    var carrier = document.querySelector('.story-share-labels');
    if (carrier) {
        var parts = carrier.querySelectorAll('p');
        if (parts[0]) labels.copy = parts[0].textContent.trim();
        if (parts[1]) labels.copied = parts[1].textContent.trim();
        carrier.parentNode.removeChild(carrier);
    }

    // Share the canonical URL rather than the address bar: that drops the
    // filter query, so a reader lands on the same page without the view state
    // of whoever shared it. The locale stays in the path, which is the point.
    var canonical = document.querySelector('link[rel="canonical"]');
    var pageUrl = (canonical && canonical.href) ||
        window.location.href.split('#')[0].split('?')[0];

    function cardUrl(card) {
        return card.id ? pageUrl + '#' + card.id : pageUrl;
    }

    function cardTitle(card) {
        var h = card.querySelector('summary h3');
        return h ? h.textContent.trim() : document.title;
    }

    var NETS = [
        { label: 'X', href: function(u, t) {
            return 'https://twitter.com/intent/tweet?text=' + encodeURIComponent(t) +
                   '&url=' + encodeURIComponent(u);
        } },
        { label: 'LinkedIn', href: function(u) {
            return 'https://www.linkedin.com/sharing/share-offsite/?url=' +
                   encodeURIComponent(u);
        } },
        { label: 'Facebook', href: function(u) {
            return 'https://www.facebook.com/sharer/sharer.php?u=' + encodeURIComponent(u);
        } }
    ];

    function copyText(text, done) {
        function fallback() {
            // Contexts without the async clipboard API, and any browser that
            // refuses it, still need a way to copy.
            var ta = document.createElement('textarea');
            ta.value = text;
            ta.setAttribute('readonly', '');
            ta.style.position = 'absolute';
            ta.style.left = '-9999px';
            document.body.appendChild(ta);
            ta.select();
            try { document.execCommand('copy'); } catch (e) {}
            document.body.removeChild(ta);
            done();
        }
        if (navigator.clipboard && navigator.clipboard.writeText) {
            navigator.clipboard.writeText(text).then(done, fallback);
            return;
        }
        fallback();
    }

    function flash(btn, text) {
        var original = btn.textContent;
        btn.textContent = text;
        btn.disabled = true;
        window.setTimeout(function() {
            btn.textContent = original;
            btn.disabled = false;
        }, 1600);
    }

    cards.forEach(function(card) {
        var row = document.createElement('div');
        row.className = 'story-share';

        NETS.forEach(function(net) {
            var a = document.createElement('a');
            a.textContent = net.label;
            a.rel = 'noopener';
            a.target = '_blank';
            a.setAttribute('data-net', net.label.toLowerCase());
            a.href = net.href(cardUrl(card), cardTitle(card));
            row.appendChild(a);
        });

        // Instagram has no web share endpoint: it takes a post from its app,
        // never a URL. The OS sheet lists it on mobile, so ask for that when
        // the browser has one, and otherwise copy the link, which is what gets
        // pasted into a story or a bio.
        var ig = document.createElement('button');
        ig.type = 'button';
        ig.textContent = 'Instagram';
        ig.setAttribute('data-net', 'instagram');
        ig.addEventListener('click', function() {
            var url = cardUrl(card);
            if (navigator.share) {
                navigator.share({ title: cardTitle(card), url: url }).catch(function() {});
                return;
            }
            copyText(url, function() { flash(ig, labels.copied); });
        });
        row.appendChild(ig);

        var cp = document.createElement('button');
        cp.type = 'button';
        cp.textContent = labels.copy;
        cp.setAttribute('data-net', 'copy');
        cp.addEventListener('click', function() {
            copyText(cardUrl(card), function() { flash(cp, labels.copied); });
        });
        row.appendChild(cp);

        // Sharing must never toggle the card: the row sits inside <summary>,
        // which is the click target for expanding it.
        row.addEventListener('click', function(e) { e.stopPropagation(); });

        var summary = card.querySelector('summary');
        if (summary) summary.appendChild(row);
    });

    // ---- deep links ----------------------------------------------------
    function openFromHash() {
        var id = window.location.hash.replace(/^#/, '');
        if (!id) return;
        var card = document.getElementById(id);
        if (!card || card.className.indexOf('story-card') === -1) return;
        // The card wins over the filter: the story a link points at may not be
        // in the group that filter would show.
        applyFilter('all', false);
        var url = new URL(window.location.href);
        if (url.searchParams.has('filter')) {
            url.searchParams.delete('filter');
            window.history.replaceState(window.history.state, '', url);
        }
        card.open = true;
        // Older browsers and non-visual DOMs (a test harness, a crawler) have
        // no scrollIntoView. Opening the card is the part that matters.
        if (typeof card.scrollIntoView === 'function') {
            card.scrollIntoView({ block: 'center' });
        }
    }

    cards.forEach(function(card) {
        card.addEventListener('toggle', function() {
            if (!card.id) return;
            var url = new URL(window.location.href);
            if (card.open) url.hash = card.id;
            else if (url.hash === '#' + card.id) url.hash = '';
            window.history.replaceState(window.history.state, '', url);
        });
    });

    openFromHash();
    window.addEventListener('hashchange', openFromHash);
});

// Language switcher: mdbook menu bar (right side).
// The book ships per-locale (English at the root, pt-PT/es/fr/ru/id in
// subdirectories). Every page exists in every locale (untranslated
// paragraphs fall back to English) so switching is a same-path
// locale-prefix swap. Works under both hosting bases: docs.opencrabs.com
// (book at root) and the landing image (book under /docs/).
document.addEventListener('DOMContentLoaded', function() {
    if (document.getElementById('lang-switcher')) return;
    var bar = document.querySelector('.menu-bar');
    if (!bar) return;

    var LOCALES = ['pt-PT', 'es', 'fr', 'ru', 'id'];
    var LABELS = { '': 'EN', 'pt-PT': 'PT', 'es': 'ES', 'fr': 'FR', 'ru': 'RU', 'id': 'ID' };

    var path = window.location.pathname;
    var base = '';
    if (path === '/docs' || path.indexOf('/docs/') === 0) {
        base = '/docs';
        path = path.slice(5) || '/';
    }
    path = path.replace(/index\.html$/, '');

    var current = '';
    var rest = path;
    for (var i = 0; i < LOCALES.length; i++) {
        var l = LOCALES[i];
        if (path === '/' + l || path.indexOf('/' + l + '/') === 0) {
            current = l;
            rest = path.slice(l.length + 1) || '/';
            break;
        }
    }

    var sel = document.createElement('select');
    sel.id = 'lang-switcher';
    sel.title = 'Language';
    sel.setAttribute('aria-label', 'Language');
    [''].concat(LOCALES).forEach(function(code) {
        var opt = document.createElement('option');
        opt.value = code;
        opt.textContent = LABELS[code];
        if (code === current) opt.selected = true;
        sel.appendChild(opt);
    });
    sel.addEventListener('change', function() {
        var next = (sel.value ? '/' + sel.value : '') + rest;
        if (next === '') next = '/';
        window.location.href = base + next;
    });

    bar.appendChild(sel);
});
