## 2026-11-25: Keep the Walls lookup independent of sprite loading

- Playwright loads the tour helpers in Node. The siege forecast imported a pure Walls-level lookup from the city art loader, pulling Vite's `import.meta.glob` into test discovery.
- Keep the lookup in the data module and re-export it for map callers; Playwright can load the formatter without loading sprites. Verify with `npx playwright test --list e2e/tour/online.spec.ts`.
