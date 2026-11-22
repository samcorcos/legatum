/** A count with its noun, agreeing in number: `counted(1, 'order')` is "1 order", `counted(0, 'order')` is "0 orders". */
export const counted = (n: number, noun: string, plural = `${noun}s`) => `${n} ${n === 1 ? noun : plural}`;
