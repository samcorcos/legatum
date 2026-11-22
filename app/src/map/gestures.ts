import type { Camera } from './render';

export type Point = { x: number; y: number };
type Size = { w: number; h: number };
export const MIN_ZOOM = 0.28;
export const MAX_ZOOM = 2.4;
export const LONG_PRESS_MS = 550;

/** Keep the world point under `from` anchored under `to`, including at the zoom limits. */
export function zoomCamera(camera: Camera, size: Size, from: Point, to: Point, factor: number): Camera {
  const zoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, camera.zoom * factor));
  return {
    zoom,
    x: camera.x + (from.x - size.w / 2) / camera.zoom - (to.x - size.w / 2) / zoom,
    y: camera.y + (from.y - size.h / 2) / camera.zoom - (to.y - size.h / 2) / zoom,
  };
}

type Contact = { id: number; point: Point; origin: Point; kind: string; button: number; pathDrag: boolean };
export type Gesture = {
  contacts: Contact[];
  phase: 'pending' | 'pan' | 'path' | 'pinch' | 'held';
  pinch?: { camera: Camera; midpoint: Point; distance: number };
};
export type GestureEvent =
  | { type: 'down'; id: number; point: Point; kind: string; button: number; pathDrag: boolean }
  | { type: 'move' | 'up'; id: number; point: Point }
  | { type: 'cancel'; id: number }
  | { type: 'hold' };
type Intent = { type: 'tap' | 'path' | 'context'; point: Point; kind: string };
export const emptyGesture = (): Gesture => ({ contacts: [], phase: 'pending' });
const gap = (a: Point, b: Point) => Math.hypot(a.x - b.x, a.y - b.y);
const midpoint = (a: Point, b: Point): Point => ({ x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 });

/** Pointer bookkeeping is independent of React, hex selection and orders. Cancellation never commits. */
export function gestureStep(state: Gesture, event: GestureEvent, camera: Camera, size: Size): { state: Gesture; camera: Camera; intent?: Intent } {
  if (event.type === 'down') {
    const contact = { ...event, origin: event.point };
    const contacts = [...state.contacts.filter((p) => p.id !== event.id), contact];
    if (contacts.length === 1) return { state: { contacts, phase: 'pending' }, camera };
    if (contacts.length === 2) {
      const [a, b] = contacts.map((p) => p.point);
      return { state: { contacts, phase: 'pinch', pinch: { camera: { ...camera }, midpoint: midpoint(a, b), distance: Math.max(1, gap(a, b)) } }, camera };
    }
    return { state: { ...state, contacts }, camera };
  }
  const first = state.contacts[0];
  if (event.type === 'hold') {
    if (state.phase !== 'pending' || state.contacts.length !== 1 || first.kind === 'mouse' || first.button !== 0) return { state, camera };
    return { state: { ...state, phase: 'held' }, camera, intent: { type: 'context', point: first.point, kind: first.kind } };
  }
  const contact = state.contacts.find((p) => p.id === event.id);
  if (!contact) return { state, camera };
  if (event.type === 'cancel') {
    // Keep other captured fingers inert until lifted, even when a third finger interrupted a pinch.
    return { state: { contacts: state.contacts.filter((p) => p.id !== event.id), phase: 'held' }, camera };
  }
  let next = { ...state, contacts: state.contacts.map((p) => p.id === event.id ? { ...p, point: event.point } : p) };
  if (state.phase === 'pinch' && state.pinch && next.contacts.length >= 2) {
    const [a, b] = next.contacts.map((p) => p.point);
    camera = zoomCamera(state.pinch.camera, size, state.pinch.midpoint, midpoint(a, b), gap(a, b) / state.pinch.distance);
  } else if (next.contacts.length === 1 && state.phase !== 'held') {
    if (state.phase === 'pending' && gap(contact.origin, event.point) > (contact.kind === 'mouse' ? 5 : 8)) {
      next.phase = contact.pathDrag && contact.kind === 'mouse' ? 'path' : 'pan';
    }
    if (next.phase === 'pan' && (contact.button === 0 || contact.button === 1)) {
      const from = state.phase === 'pending' ? contact.origin : contact.point;
      camera = { ...camera, x: camera.x - (event.point.x - from.x) / camera.zoom, y: camera.y - (event.point.y - from.y) / camera.zoom };
    }
  }
  if (event.type !== 'up') return { state: next, camera };
  const intent: Intent | undefined = next.phase === 'pending' && contact.button === 0 ? { type: 'tap', point: event.point, kind: contact.kind }
    : next.phase === 'path' ? { type: 'path', point: event.point, kind: contact.kind } : undefined;
  next = { contacts: next.contacts.filter((p) => p.id !== event.id), phase: 'held' };
  return { state: next.contacts.length ? next : emptyGesture(), camera, intent };
}

export type Tap = { point: Point; time: number };
export function isDoubleTap(previous: Tap | null, next: Tap): boolean {
  return !!previous && next.time >= previous.time && next.time - previous.time <= 300 && gap(previous.point, next.point) <= 24;
}
