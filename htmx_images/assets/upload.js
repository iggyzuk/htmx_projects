htmx.on('#img-upload-form', 'htmx:xhr:progress', function(evt) {
    var loaded = evt.detail.loaded;
    var total = evt.detail.total;
    var progress = (loaded / total) * 100;

    document.getElementById('progress').style.width = progress + '%';
});

htmx.on('htmx:afterRequest', function(evt) {
    if (evt.detail.elt && evt.detail.elt.id === 'img-upload-form') {
        document.getElementById('progress').style.width = '0%'; // Reset progress bar
        document.getElementById('form-file').value = ''; // Clear file input field
    }
});