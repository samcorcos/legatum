export function randomSeed(): string {
  // Up to 2^53 so the decimal string is exact; the core takes any u64.
  const hi = Math.floor(Math.random() * 2 ** 21);
  const lo = Math.floor(Math.random() * 2 ** 32);
  return String(hi * 2 ** 32 + lo);
}
