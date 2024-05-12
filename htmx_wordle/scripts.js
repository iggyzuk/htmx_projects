document.addEventListener('dblclick', function (event) {
    event.preventDefault();
}, { passive: false });

document.addEventListener('alpine:init', () => {
    Alpine.data('wordleDataObject', () => ({
        letters: [],
        combine() { return this.letters.join(''); },
        fill() { return this.letters.concat(Array(5 - this.letters.length).fill('-')).join(''); },
        addLetter(value) {
            const letter = value.toLowerCase();
            if (letter.length === 1 && this.letters.length < 5) {
                this.letters.push(letter);
            }
        },
        removeLetter() {
            if (this.letters.length > 0) {
                this.letters.pop();
            }
        },
        clear() {
            this.letters = [];
        }
    }));
});

// HTMX history cache and Alpine template tags not working together #2924
// https://github.com/alpinejs/alpine/discussions/2924
document.addEventListener('htmx:beforeHistorySave', (evt) => {
    document.querySelectorAll('[x-from-template]').forEach((e) => e.remove());
})