import { stepReport, toggleLastTurnReport, useGame } from '../store';
import { gameplayKeyAllowed } from '../gameplayKeys';
import { readyAndPass } from '../ui/SeatBar';
import { pendingDecisions } from '../turnFlow';
import { useAdvisor } from '../ui/advisorModel';
import { toggleOverlay } from '../map/overlays';
import { airState, rebaseOptions, reconBlocker } from '../airWing';
import { cityAutomationFocus, lastAutomationFocus, nextAutomationFocus, setCityAutomation } from '../automation';
import {
  toggleAutoExploreSelected,
  cancelSelected,
  disbandSelected,
  foundCitySelected,
  sleepSelected,
  healSelected,
  toggleAutoFireSelected,
  skipSelected,
  toggleRangedMode,
  upgradeSelected,
} from '../commands';

/**
 * Aircraft keys: with an aircraft selected, Space skips, F waits on guard, S opens Strike and R opens Rebase.
 * Installed on window in the capture phase after `handleOverlayKey`, so these four keys beat
 * map panning (S) and the ground ranged attack and Fortify (R, F) without moving the rest of the game keys out of the
 * bubble phase, where panels and celebrations still consume their keys first.
 */
export function handleAircraftKey(e: KeyboardEvent) {
  if (!gameplayKeyAllowed(e) || e.repeat) return;
  const s = useGame.getState();
  const view = s.views?.[s.seat];
  const selectedPlane = s.selection?.kind === 'air' ? view?.aircraft?.find((plane) => plane.id === s.selection!.id && plane.owner === view.me) : undefined;
  const selectedCity = selectedPlane ? view?.cities.find((city) => city.id === selectedPlane.city) : undefined;
  if (view && selectedCity && selectedPlane) {
    const orders = s.orders[s.seat] ?? [];
    const errors = s.errors[s.seat] ?? [];
    const validOrders = orders.filter((_, index) => !errors.some((error) => error.index === index));
    const plane = selectedPlane;
    if (plane) {
      const key = e.key.toLowerCase();
      const hasOrder = validOrders.some((order) => 'plane' in order && order.plane === plane.id);
      if (key === ' ' || key === 'f') {
        e.preventDefault();
        e.stopPropagation();
        if (!hasOrder) {
          const state = airState(view, selectedCity, plane, validOrders, s.airHeld[view.me]);
          (key === 'f' && state.kind !== 'ready' ? s.sleepAir : s.holdAir)(plane.id);
        }
        return;
      }
      if (key === 's') {
        e.preventDefault();
        e.stopPropagation();
        if (!hasOrder) s.set({ air: { plane: plane.id, what: s.air.plane === plane.id && s.air.what === 'strike' ? null : 'strike', hover: null } });
        return;
      }
      if (key === 'a') {
        e.preventDefault();
        e.stopPropagation();
        if (!hasOrder && !reconBlocker(view, selectedCity, plane, orders)) {
          s.set({ air: { plane: plane.id, what: s.air.plane === plane.id && s.air.what === 'recon' ? null : 'recon', hover: null } });
        }
        return;
      }
      if (key === 'r') {
        e.preventDefault();
        e.stopPropagation();
        if (!hasOrder && rebaseOptions(view, selectedCity, orders, errors).some((option) => option.available)) {
          s.set({ air: { plane: plane.id, what: s.air.plane === plane.id && s.air.what === 'rebase' ? null : 'rebase', hover: null } });
        }
        return;
      }
    }
  }
}

export function handleGameKey(e: KeyboardEvent) {
  if (!gameplayKeyAllowed(e) || e.repeat) return;
  const s = useGame.getState();
  // Choosing a build tile is the only decision until the map receives a tile or Esc cancels.
  if (s.place && e.key !== 'Escape') return;
  switch (e.key) {
    case ' ':
      e.preventDefault();
      skipSelected();
      break;
    case 'Enter': {
      e.preventDefault();
      if (document.querySelector('[data-advisor-id="welcome"]')) { useAdvisor.getState().markSeen('welcome'); break; }
      const finished = s.views?.[s.seat];
      // A finished game's one action: reopen its report.
      if (finished?.finished) { if (finished.history) s.openGameReport(); break; }
      if (e.shiftKey) readyAndPass();
      else if (!s.ready[s.seat]) {
        const view = s.views?.[s.seat];
        if (view && pendingDecisions(view, s.orders[s.seat], s.errors[s.seat], s).length) s.nextDecision();
        else if (view) readyAndPass();
      }
      break;
    }
    case 'Escape': {
      // From a keyboard-focused control, Escape first hands focus back to the map so Return works again.
      const map = document.querySelector<HTMLElement>('[data-game-map]');
      if (map && e.target !== map && e.target !== document.body) { map.focus({ preventScroll: true }); break; }
      // Escape peels one layer at a time: the turn report first, then targeting, then selection.
      if (s.reportOpen && s.report) s.set({ reportOpen: false });
      else if (s.pingMode) s.set({ pingMode: false });
      else if (s.place) s.set({ place: null });
      else if (s.air.what) s.set({ air: { plane: null, what: null, hover: null } });
      else if (s.mode !== 'normal') s.setMode('normal');
      else s.select(null);
      break;
    }
    case '1':
    case 'e':
    case 'E':
      toggleAutoExploreSelected();
      break;
    case 'i':
    case 'I': {
      const view = s.views?.[s.seat];
      const city = s.selection?.kind === 'city' ? view?.cities.find((c) => c.id === s.selection?.id && c.owner === view.me) : undefined;
      if (view && city) {
        e.preventDefault();
        const focus = cityAutomationFocus(city.id, view, s.orders[s.seat]);
        s.editOrders((orders) => setCityAutomation(orders, city.id, e.shiftKey ? nextAutomationFocus(focus) : focus ? null : lastAutomationFocus()));
      }
      break;
    }
    case 'f':
    case 'F':
      sleepSelected();
      break;
    case 'h':
    case 'H':
      healSelected();
      break;
    case 'b':
    case 'B':
      foundCitySelected();
      break;
    case 'r':
    case 'R':
      toggleRangedMode();
      break;
    case 'v':
    case 'V':
      toggleAutoFireSelected();
      break;
    case 'u':
    case 'U':
      upgradeSelected();
      break;
    case 't':
    case 'T':
      s.set({ techOpen: true });
      break;
    case 'c':
    case 'C':
      s.set({ citiesOpen: !s.citiesOpen });
      break;
    case 'l':
    case 'L':
      toggleLastTurnReport();
      break;
    // Map overlays (R is the ranged attack, so Resources takes O).
    case 'y':
    case 'Y':
      toggleOverlay('yields');
      break;
    case 'o':
    case 'O':
      toggleOverlay('resources');
      break;
    case 'n':
    case 'N':
      toggleOverlay('borders');
      break;
    case '[':
    case ']':
      if (s.reportOpen && s.report) stepReport(e.key === '[' ? -1 : 1);
      break;
    case 'Backspace':
    case 'Delete':
      e.preventDefault();
      cancelSelected();
      break;
    case 'x':
    case 'X':
      if (e.shiftKey) disbandSelected();
      break;
    case 'p':
    case 'P':
      if (!e.ctrlKey && !e.metaKey && !e.altKey && s.canPing()) s.setPingMode(!s.pingMode);
      break;
  }
}
