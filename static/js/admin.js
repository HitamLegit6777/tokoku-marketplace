// Admin panel interactions: sidebar toggle for mobile.
(function () {
  'use strict';
  var sidebar = document.getElementById('adminSidebar');
  var btn = document.getElementById('adminMenuBtn');
  var backdrop = document.getElementById('adminBackdrop');
  function toggle() {
    if (!sidebar) return;
    sidebar.classList.toggle('open');
    if (backdrop) backdrop.classList.toggle('show', sidebar.classList.contains('open'));
  }
  function close() {
    if (sidebar) sidebar.classList.remove('open');
    if (backdrop) backdrop.classList.remove('show');
  }
  if (btn) btn.addEventListener('click', toggle);
  if (backdrop) backdrop.addEventListener('click', close);
})();

// Search and filter the contextual theme gallery.
(function () {
  'use strict';
  var search = document.getElementById('themeSearch');
  var filters = document.getElementById('themeFilters');
  var cards = Array.prototype.slice.call(document.querySelectorAll('.admin-theme-picker .theme-option-form'));
  if (!search || !cards.length) return;
  var layout = 'all';
  function apply() {
    var query = search.value.trim().toLowerCase();
    cards.forEach(function (card) {
      var matchesLayout = layout === 'all' || card.dataset.layout === layout;
      var matchesText = !query || (card.dataset.search || '').toLowerCase().indexOf(query) !== -1;
      card.hidden = !(matchesLayout && matchesText);
    });
  }
  search.addEventListener('input', apply);
  if (filters) filters.addEventListener('click', function (event) {
    var button = event.target.closest('[data-layout]');
    if (!button) return;
    filters.querySelectorAll('.theme-filter').forEach(function (item) { item.classList.remove('active'); });
    button.classList.add('active');
    layout = button.dataset.layout;
    apply();
  });
})();

// Inline edit for split-layout admin lists (categories, banners, coupons).
// A table/list "Edit" button carries data-edit='{"field":"value",...}'.
// It fills the target form (by [data-edit-form]) whose fields match those
// names, sets the hidden id, and flips the form into "edit mode".
(function () {
  'use strict';
  var form = document.querySelector('[data-edit-form]');
  if (!form) return;

  var idInput = form.querySelector('input[name="id"]');
  var title = form.querySelector('[data-edit-title]');
  var submit = form.querySelector('[type="submit"]');
  var cancel = form.querySelector('[data-edit-cancel]');
  var createTitle = title ? title.textContent : '';
  var createLabel = submit ? submit.textContent : '';
  var editTitle = form.getAttribute('data-edit-title-text') || 'Edit';
  var editLabel = form.getAttribute('data-edit-label') || 'Simpan Perubahan';

  function fill(data) {
    Object.keys(data).forEach(function (name) {
      var field = form.elements[name];
      if (!field || name === 'id') return;
      field.value = data[name] == null ? '' : data[name];
    });
    if (idInput) idInput.value = data.id || '';
    if (title) title.textContent = editTitle;
    if (submit) submit.textContent = editLabel;
    if (cancel) cancel.hidden = false;
    form.classList.add('is-editing');
    var first = form.querySelector('input:not([type=hidden]), select, textarea');
    if (first) first.focus();
    form.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }

  function reset() {
    form.reset();
    if (idInput) idInput.value = '';
    if (title) title.textContent = createTitle;
    if (submit) submit.textContent = createLabel;
    if (cancel) cancel.hidden = true;
    form.classList.remove('is-editing');
  }

  document.addEventListener('click', function (e) {
    var t = e.target.closest('[data-edit]');
    if (!t) return;
    e.preventDefault();
    try { fill(JSON.parse(t.getAttribute('data-edit'))); }
    catch (err) { /* ignore malformed payload */ }
  });

  if (cancel) cancel.addEventListener('click', function (e) { e.preventDefault(); reset(); });
})();
