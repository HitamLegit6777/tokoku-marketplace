// Reusable image upload helper. Handles the product form drop zone and any
// [data-target] upload trigger buttons that fill a text input with the URL.
(function () {
  'use strict';

  function upload(file) {
    var fd = new FormData();
    fd.append('image', file);
    return fetch('/admin/upload', { method: 'POST', body: fd }).then(function (r) { return r.json(); });
  }

  // Product form multi-image uploader
  var input = document.getElementById('imageInput');
  var drop = document.getElementById('imageDrop');
  var list = document.getElementById('imageList');
  var field = document.getElementById('imagesField');

  function syncField() {
    if (!field || !list) return;
    var urls = [];
    list.querySelectorAll('img').forEach(function (img) { urls.push(img.src.replace(window.location.origin, '')); });
    field.value = urls.join('\n');
  }

  function addThumb(url) {
    if (!list) return;
    var div = document.createElement('div');
    div.className = 'image-thumb';
    div.innerHTML = '<img src="' + url + '" alt=""><button type="button" class="img-remove" onclick="removeImage(this)">&times;</button>';
    list.appendChild(div);
    syncField();
  }

  function handleFiles(files) {
    Array.prototype.forEach.call(files, function (f) {
      if (!f.type.indexOf) return;
      if (drop) { drop.classList.add('drag'); var span = drop.querySelector('span'); if (span) span.textContent = 'Mengupload...'; }
      upload(f).then(function (res) {
        if (res.success) addThumb(res.url);
        else alert('Upload gagal: ' + (res.error || ''));
        if (drop) { drop.classList.remove('drag'); var span = drop.querySelector('span'); if (span) span.textContent = 'Klik untuk upload gambar'; }
      });
    });
  }

  if (input) input.addEventListener('change', function () { handleFiles(this.files); this.value = ''; });
  if (drop) {
    ['dragover', 'dragenter'].forEach(function (ev) { drop.addEventListener(ev, function (e) { e.preventDefault(); drop.classList.add('drag'); }); });
    ['dragleave', 'drop'].forEach(function (ev) { drop.addEventListener(ev, function (e) { e.preventDefault(); drop.classList.remove('drag'); }); });
    drop.addEventListener('drop', function (e) { if (e.dataTransfer && e.dataTransfer.files) handleFiles(e.dataTransfer.files); });
  }
  // keep field in sync if user pastes urls manually
  if (field) field.addEventListener('input', function () {});

  window.removeImage = function (btn) {
    var thumb = btn.closest('.image-thumb');
    if (thumb) thumb.remove();
    syncField();
  };

  // Single-target upload triggers (settings, payment, banners)
  document.querySelectorAll('.upload-trigger').forEach(function (btn) {
    btn.addEventListener('click', function () {
      var targetId = btn.dataset.target;
      var picker = document.createElement('input');
      picker.type = 'file';
      picker.accept = 'image/*';
      picker.onchange = function () {
        if (!picker.files[0]) return;
        var old = btn.textContent; btn.textContent = 'Mengupload...';
        upload(picker.files[0]).then(function (res) {
          btn.textContent = old;
          if (res.success) {
            var target = document.getElementById(targetId);
            if (target) target.value = res.url;
          } else alert('Upload gagal: ' + (res.error || ''));
        });
      };
      picker.click();
    });
  });
})();
