document.addEventListener("htmx:confirm", function (e) {
    // Check if the target element has hx-confirm attribute
    if (!e.target.matches("[hx-confirm]")) {
        return;
    }

    e.preventDefault();

    // Set modal content
    var modalText = document.getElementById("confirm-modal-text");
    modalText.innerText = `${e.detail.question}`;

    // Show modal
    var confirmModal = document.getElementById("confirm-modal");
    var modal = new bootstrap.Modal(confirmModal);
    modal.show();

    // Handle all close buttons
    var closeModalButtons = document.querySelectorAll("[data-dismiss='modal']");
    closeModalButtons.forEach(function (button) {
        modal.hide();
    });

    // Handle proceed button click
    var proceedButton = document.getElementById("confirm-modal-proceed");
    proceedButton.addEventListener("click", function () {
        modal.hide();
        e.detail.issueRequest(true); // use true to skip window.confirm
    });
});