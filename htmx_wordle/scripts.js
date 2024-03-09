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
            console.log("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
            this.letters = [];
        }
    }));
});