/** System reduced motion and the in-game Map animations switch both mean an immediate final state. */
export const motionStill = (reducedMotion: boolean, animationsOn: boolean): boolean => reducedMotion || !animationsOn;
