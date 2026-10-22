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

// User Stories — Hermes-style category filter pills.
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

    pills.forEach(function(pill) {
        pill.addEventListener('click', function() {
            var f = pill.getAttribute('data-filter');
            pills.forEach(function(p) { p.classList.toggle('active', p === pill); });
            cards.forEach(function(card) {
                var show = (f === 'all') || (cardGroup(card) === f);
                card.classList.toggle('is-hidden', !show);
            });
        });
    });
});

// Language switcher — mdbook menu bar (right side).
// The book ships per-locale (English at the root, pt-PT/es/fr/ru/id in
// subdirectories). Every page exists in every locale — untranslated
// paragraphs fall back to English — so switching is a same-path
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
