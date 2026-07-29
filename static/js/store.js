// Storefront interactions: mobile drawer, header shadow on scroll, image gallery,
// quantity stepper, clipboard copy, and live cart badge refresh.
(function () {
  'use strict';

  // Mobile drawer
  var drawer = document.getElementById('mobileDrawer');
  var openBtn = document.getElementById('menuToggle');
  var closeBtn = document.getElementById('drawerClose');
  var backdrop = document.getElementById('drawerBackdrop');
  function openDrawer() { if (drawer) drawer.classList.add('open'); document.body.style.overflow = 'hidden'; }
  function closeDrawer() { if (drawer) drawer.classList.remove('open'); document.body.style.overflow = ''; }
  if (openBtn) openBtn.addEventListener('click', openDrawer);
  if (closeBtn) closeBtn.addEventListener('click', closeDrawer);
  if (backdrop) backdrop.addEventListener('click', closeDrawer);

  // Header shadow on scroll
  var header = document.getElementById('siteHeader');
  if (header) {
    window.addEventListener('scroll', function () {
      header.classList.toggle('scrolled', window.scrollY > 8);
    }, { passive: true });
  }

  // Highlight active bottom-nav item
  var path = window.location.pathname;
  document.querySelectorAll('.bn-item').forEach(function (a) {
    var href = a.getAttribute('href');
    if (href === path || (href !== '/' && path.indexOf(href) === 0)) a.classList.add('active');
    else if (href === '/' && path === '/') a.classList.add('active');
  });

  // Refresh cart badge from API (keeps header in sync across tabs)
  fetch('/cart/count').then(function (r) { return r.json(); }).then(function (d) {
    var badge = document.getElementById('cartBadge');
    if (badge) {
      if (d.count > 0) { badge.textContent = d.count; badge.style.display = ''; }
      else { badge.style.display = 'none'; }
    }
  }).catch(function () {});
})();

// Product gallery
function setMainImage(src, el) {
  var main = document.getElementById('pdMainImg');
  if (main) main.src = src;
  document.querySelectorAll('.pd-thumb').forEach(function (t) { t.classList.remove('active'); });
  if (el) el.classList.add('active');
}

// Quantity stepper on product detail
function changeQty(delta) {
  var input = document.getElementById('qtyInput');
  if (!input) return;
  var val = parseInt(input.value || '1', 10) + delta;
  var max = input.max ? parseInt(input.max, 10) : Infinity;
  if (val < 1) val = 1;
  if (val > max) val = max;
  input.value = val;
}

// Clipboard copy with tiny toast
function copyText(text) {
  navigator.clipboard.writeText(text).then(function () { showToast('Disalin!'); }).catch(function () {});
}
function showToast(msg) {
  var t = document.createElement('div');
  t.textContent = msg;
  t.style.cssText = 'position:fixed;bottom:90px;left:50%;transform:translateX(-50%);background:#111;color:#fff;padding:10px 18px;border-radius:100px;font-size:13px;z-index:999;font-weight:600;box-shadow:0 6px 20px rgba(0,0,0,.3)';
  document.body.appendChild(t);
  setTimeout(function () { t.style.transition = 'opacity .3s'; t.style.opacity = '0'; setTimeout(function () { t.remove(); }, 300); }, 1400);
}
