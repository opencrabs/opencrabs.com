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

// User Stories: filter pills and a URL that keeps up.
//
// The pills used to only toggle a class, so a filtered wall had no address to
// copy. The filter is now ?filter=<slug>, pushed on click and read on load, so
// a link opens the same view and the back button walks the filters.
//
// Each card links to its own page (stories/<slug>.html). The card keeps a
// stable id (story-<slug>, derived from the English title so it is the same
// string in all six books) only so shares made before the pages existed still
// resolve: a #story-<slug> link is forwarded to the page it now lives on.
//
// ---- share rows ------------------------------------------------------
// Share lives on the story page, not on the wall. The wall used to build a row
// of five links per card, which is 115 controls of small text on one page, all
// pointing at the same five places and none of them telling a reader which
// story they were about to share. The page is the thing worth passing on, so
// the row is built there and the wall stays a wall.
//
// The two labels are markdown, in a hidden carrier, so the five catalogs can
// translate them. Read the carrier once and drop it: it exists for the
// extractor, not for the page.
function shareLabels() {
    var labels = { copy: 'Copy link', copied: 'Copied' };
    var carrier = document.querySelector('.story-share-labels');
    if (carrier) {
        var parts = carrier.querySelectorAll('p');
        if (parts[0]) labels.copy = parts[0].textContent.trim();
        if (parts[1]) labels.copied = parts[1].textContent.trim();
        carrier.parentNode.removeChild(carrier);
    }
    return labels;
}

var SHARE_NETS = [
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

function buildShareRow(url, title, labels) {
    var row = document.createElement('div');
    row.className = 'story-share';

    SHARE_NETS.forEach(function(net) {
        var a = document.createElement('a');
        a.textContent = net.label;
        a.rel = 'noopener';
        a.target = '_blank';
        a.setAttribute('data-net', net.label.toLowerCase());
        a.href = net.href(url, title);
        row.appendChild(a);
    });

    // Instagram has no web share endpoint: it takes a post from its app, never
    // a URL. The OS sheet lists it on mobile, so ask for that when the browser
    // has one, and otherwise copy the link, which is what gets pasted into a
    // story or a bio.
    var ig = document.createElement('button');
    ig.type = 'button';
    ig.textContent = 'Instagram';
    ig.setAttribute('data-net', 'instagram');
    ig.addEventListener('click', function() {
        if (navigator.share) {
            navigator.share({ title: title, url: url }).catch(function() {});
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
        copyText(url, function() { flash(cp, labels.copied); });
    });
    row.appendChild(cp);

    return row;
}

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

    // ---- old fragment links --------------------------------------------
    // Shares made before the stories had their own pages point at the wall
    // with a story-<slug> fragment on the end. Those links are out in the
    // world, so send them to the story's page instead of leaving the reader at
    // the top of a wall.
    function followLegacyFragment() {
        var id = window.location.hash.replace(/^#/, '');
        if (!id) return false;
        var card = document.getElementById(id);
        if (!card) return false;
        var a = card.querySelector('.story-link');
        if (!a) return false;
        window.location.replace(a.href);
        return true;
    }

    if (!followLegacyFragment()) {
        window.addEventListener('hashchange', followLegacyFragment);
    }
});

// The share row a story page offers. The page is plain markdown, so there is
// no host element to append to: the row goes right after the author line,
// which is the last piece of page chrome before the prose, so it reads as part
// of the byline instead of interrupting the story.
//
// Guarded against the wall: a page holding cards is not a story page.
document.addEventListener('DOMContentLoaded', function() {
    if (document.querySelector('.story-card')) return;
    var author = document.querySelector('.story-author');
    if (!author) return;

    // Share the title the reader sees, not the <title> mdbook builds, which
    // carries the book name on the end.
    var h1 = document.querySelector('.content main h1');
    var title = (h1 ? h1.textContent.trim() : '') || document.title;
    var url = window.location.href.split('#')[0];
    var row = buildShareRow(url, title, shareLabels());
    row.className = 'story-share story-share-page';
    author.parentNode.insertBefore(row, author.nextSibling);
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
