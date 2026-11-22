/**
 * The hosted web build (scripts/build-web.sh sets VITE_LEGATUM_HOSTED=1): static files on a web host, with no
 * legatum-server behind them. It never asks for /api/status, so nothing fails or waits on a server that is not
 * there, and the menus say that network games need the game run from its source code.
 */
export const HOSTED: boolean = import.meta.env.VITE_LEGATUM_HOSTED === '1';

/** Where the hosted build sends people who want a network game. */
export const SOURCE_URL = 'https://github.com/samcorcos/legatum#play-it';
