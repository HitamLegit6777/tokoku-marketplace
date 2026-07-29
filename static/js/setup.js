// Setup wizard step navigation with basic per-step validation.
(function () {
  'use strict';
  var current = 1;
  var total = 4;
  function show(step) {
    document.querySelectorAll('.setup-step').forEach(function (el) {
      el.classList.toggle('active', parseInt(el.dataset.step, 10) === step);
    });
    document.querySelectorAll('.sp-step').forEach(function (el) {
      var s = parseInt(el.dataset.step, 10);
      el.classList.toggle('active', s === step);
      el.classList.toggle('done', s < step);
    });
    current = step;
    window.scrollTo({ top: 0, behavior: 'smooth' });
  }
  window.nextStep = function () {
    // Validate required fields in the current step
    var stepEl = document.querySelector('.setup-step[data-step="' + current + '"]');
    if (stepEl) {
      var invalid = null;
      stepEl.querySelectorAll('[required]').forEach(function (inp) {
        if (!invalid && !inp.value.trim()) invalid = inp;
      });
      if (invalid) { invalid.focus(); invalid.style.borderColor = '#dc2626'; return; }
    }
    if (current < total) show(current + 1);
  };
  window.prevStep = function () { if (current > 1) show(current - 1); };
})();
