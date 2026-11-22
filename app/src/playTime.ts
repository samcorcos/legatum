interface ArchivedTurnTime {
  turn: number;
  ended_at: string;
  active_seconds: number;
  era: number;
  session_id: string;
}

export interface PlayTimeSummary {
  turns: number;
  seconds: number;
  sessions: number;
  minutesPerTurn: number;
}

export function playTimeSummary(archiveJson?: string): PlayTimeSummary | null {
  if (!archiveJson) return null;
  try {
    const archive = JSON.parse(archiveJson) as { turn_times?: ArchivedTurnTime[] };
    const turns = archive.turn_times ?? [];
    if (!turns.length) return null;
    const seconds = turns.reduce((sum, turn) => sum + turn.active_seconds, 0);
    return {
      turns: turns.length,
      seconds,
      sessions: new Set(turns.map((turn) => turn.session_id)).size,
      minutesPerTurn: seconds / turns.length / 60,
    };
  } catch { return null; }
}

export function formatPlayDuration(seconds: number) {
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  if (hours) return `${hours} h ${remainingMinutes} min`;
  return `${minutes} min`;
}

export function playTimeReportLine(summary: PlayTimeSummary) {
  return `Played: ${formatPlayDuration(summary.seconds)} over ${summary.sessions} ${summary.sessions === 1 ? 'session' : 'sessions'} · about ${summary.minutesPerTurn.toFixed(1)} min a turn`;
}

export function archivePlaySeconds(archiveJson?: string) {
  return playTimeSummary(archiveJson)?.seconds ?? 0;
}
