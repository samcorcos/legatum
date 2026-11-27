import type { Faction } from '../types';

/**
 * How each people greets an empire it meets for the first time (celebrate/MeetingTakeover.tsx). `native` is a real
 * greeting or formula of that people, in their own language and written in the Latin alphabet where a player could
 * not read the original; `meaning` is its plain English; `line` is what they say to you, in their voice. Every native
 * line is attested; a people whose own words we cannot check gets none (the plate then shows only `line`).
 *
 * Sources for the native lines: Latin "salvete"; Greek "chairete" (the Athenian and Spartan greeting, Doric spelling
 * aside); "eirene hymin", the Greek Christian greeting of the Byzantine court and liturgy; the Egyptian letter formula
 * ankh wedja seneb ("life, prosperity, health") after a king's name; Nahuatl "nimitztlahpaloa" (tlahpaloa, to greet,
 * in Molina's 1571 dictionary); the opening of Guyuk Khan's 1246 letter to Pope Innocent IV, "by the power of
 * Eternal Heaven"; Old Irish "failte" (welcome); German "seid gegruesst"; Sanskrit "namaste". Nahuatl
 * "nimitztlahpaloa" is the dictionary verb, not a recorded court formula. Switzerland has no native line: "grüezi" is
 * first recorded in the 19th century.
 */
export interface Greeting { native?: string; meaning?: string; line: string }

export const GREETING: Record<Faction, Greeting> = {
  Rome: { native: 'Salvete.', meaning: 'Greetings.', line: 'The Senate and People of Rome greet you. Our roads run far. Let them bring friends to your gates.' },
  Athens: { native: 'Chaírete.', meaning: 'Be well.', line: 'Athens greets you. Come and hear our assembly and our poets. We have much to teach.' },
  Sparta: { native: 'Chaírete.', meaning: 'Be well.', line: 'Sparta sees you. Be a good neighbour.' },
  Egypt: { native: 'Ankh, wedja, seneb.', meaning: 'Life, prosperity, health.', line: 'Pharaoh greets you from the Two Lands. The river feeds us all. Let it feed friendship between us.' },
  Byzantium: { native: 'Eirēnē hymîn.', meaning: 'Peace be with you.', line: 'The Emperor in Constantinople greets you. The City has outlasted many neighbours. Be a friendly one.' },
  Aztecs: { native: 'Nimitztlahpaloa.', meaning: 'I greet you.', line: 'Tenochtitlan greets you. Honour the Mexica, and you will be our friends.' },
  Mongols: { native: 'Möngke Tengri-yin küchün-dür.', meaning: 'By the power of Eternal Heaven.', line: 'The Great Khan has heard of you. Those who bow are spared. Those who resist are remembered.' },
  Celts: { native: 'Fáilte.', meaning: 'Welcome.', line: 'The tribes have seen your fires. A guest is sacred at our hearths. A raider is not.' },
  Germany: { native: 'Seid gegrüßt.', meaning: 'Be greeted.', line: 'The German Empire greets you. Our forges never rest. Let us be neighbours, not rivals.' },
  Switzerland: { line: 'The Confederates greet you. Come as a guest and the passes are open. Come as an army and they are not.' },
  India: { native: 'Namaste.', meaning: 'I bow to you.', line: 'The court of Hindustan greets you. Many peoples live under one emperor here. There is room for one more friend.' },
};

/** Short lines spoken by the other people at the peace table and on receiving a declaration. */
export const PEACE_OFFER: Record<Faction, string> = {
  Rome: 'Let the Senate set its seal to peace, and our soldiers return to their fields.',
  Athens: 'Let reason settle what spears could not. We offer a peace both cities can keep.',
  Sparta: 'Enough. Let each people hold what is theirs, and put down the spear.',
  Egypt: 'The river has carried enough grief. Let our lands prosper in peace.',
  Byzantium: 'A wise ruler knows when to close a war. Let our envoys finish it.',
  Aztecs: 'The tribute of battle has been paid. Let our peoples lay down their shields.',
  Mongols: 'The road ahead is long. Let our riders leave your lands in peace.',
  Celts: 'Let the fires burn for guests again, and no longer for war.',
  Germany: 'Our princes have spent enough on war. Let the borders stand in peace.',
  Switzerland: 'Our cantons ask for quiet in the valleys. Let us agree to peace.',
  India: 'Let the court record an end to this war, and our people tend their fields.',
};

export const WAR_REPLY: Record<Faction, string> = {
  Rome: 'The Senate will remember who crossed the border first.',
  Athens: 'Our assembly will answer your march with ships and spears.',
  Sparta: 'Come, then. We will meet you in the field.',
  Egypt: 'The Two Lands will stand before your army.',
  Byzantium: 'The walls of the City have heard such threats before.',
  Aztecs: 'Our warriors will meet yours beneath their banners.',
  Mongols: 'Our riders will carry word of your choice across the steppe.',
  Celts: 'You will find every hearth defended.',
  Germany: 'Our princes will answer your declaration together.',
  Switzerland: 'The passes are narrow, and we know every one.',
  India: 'The court has heard your challenge. Our armies will answer.',
};
