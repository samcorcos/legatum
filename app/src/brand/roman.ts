// Roman numerals for eras and Ages. Kept free of asset imports so Node-side code (the screen tour's helpers) can load it.
const ROMAN = ['I', 'II', 'III', 'IV', 'V', 'VI', 'VII', 'VIII', 'IX', 'X', 'XI', 'XII'];
export const roman = (n: number) => ROMAN[n - 1] ?? String(n);
