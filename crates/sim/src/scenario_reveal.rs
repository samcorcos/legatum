//! One-off repair for scenario positions cut before rules 25 changed strategic reveals.
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::{GameState, Order, PlayerId, TechId, player_view};
use legatum_wasm::scenario::{ScenarioFile, encode_state};
use legatum_wasm::session::brain_seed;
use std::path::Path;

const MOVED: [(TechId, TechId); 4] = [
    (TechId::Education, TechId::Gunpowder),
    (TechId::Metallurgy, TechId::Industrialization),
    (TechId::Industrialization, TechId::Combustion),
    (TechId::Electricity, TechId::AutonomousSystems),
];

/// Grant only reveals that this empire could already see under the old rules.
/// Return the affected empire names and granted technologies.
pub fn migrate(state: &mut GameState) -> Vec<String> {
    let mut edits = Vec::new();
    for index in 0..state.players.len() {
        let id = PlayerId(index as u8);
        let grants: Vec<TechId> = MOVED
            .iter()
            .filter(|(old, new)| {
                state.player(id).techs.contains(old) && !state.player(id).techs.contains(new)
            })
            .map(|(_, new)| *new)
            .collect();
        if grants.is_empty() {
            continue;
        }
        let was_researching = state.player(id).researching;
        {
            let player = state.player_mut(id);
            player.techs.extend(grants.iter().copied());
            player
                .research_queue
                .retain(|tech| !player.techs.contains(tech));
            if was_researching.is_some_and(|tech| grants.contains(&tech)) {
                player.researching = player.research_queue.first().copied();
            }
        }
        if was_researching.is_some_and(|tech| grants.contains(&tech))
            && state.player(id).researching.is_none()
        {
            // Use the same AI research picker used on the next planning turn. The stored
            // science is left intact, becoming progress toward the replacement choice.
            let player = state.player(id);
            let mut brain = ScriptedBrain::with_difficulty(
                player.faction,
                brain_seed(state.setup.seed, id),
                id,
                state.setup.players[index].difficulty,
            );
            let choice = brain
                .orders(&player_view(state, id))
                .into_iter()
                .find_map(|order| match order {
                    Order::SetResearchQueue { techs, .. } => techs.first().copied(),
                    Order::SetResearch { tech, .. } => Some(tech),
                    _ => None,
                });
            if let Some(tech) = choice {
                let player = state.player_mut(id);
                player.researching = Some(tech);
                player.research_queue = vec![tech];
            }
        }
        edits.push(format!(
            "{:?}: {}",
            state.player(id).faction,
            grants
                .iter()
                .map(|tech| format!("{tech:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    edits
}

pub fn migrate_file(path: &Path) -> Result<Option<Vec<String>>, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut file = ScenarioFile::parse(&raw)?;
    let mut state = file.stored_state()?;
    let edits = migrate(&mut state);
    if edits.is_empty() {
        return Ok(None);
    }
    file.state = encode_state(&state);
    file.revision += 1;
    file.baseline = None;
    let text = serde_json::to_string_pretty(&file).map_err(|e| e.to_string())? + "\n";
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Some(edits))
}

pub fn main(args: Vec<String>) -> Result<(), String> {
    if args.len() < 2 {
        return Err("usage: legatum-sim scenario-reveal-migrate SCENARIO.json...".into());
    }
    for arg in args.iter().skip(1) {
        match migrate_file(Path::new(arg))? {
            Some(edits) => println!("{arg}: {}", edits.join("; ")),
            None => println!("{arg}: unchanged"),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use legatum_core::{
        Faction, GameSetup, new_game,
        resources::{self, Resource},
    };

    #[test]
    fn current_research_advances_and_keeps_progress() {
        let mut state = new_game(&GameSetup::all_ai(17));
        let id = state.players[0].id;
        let player = state.player_mut(id);
        player.techs.insert(TechId::Education);
        player.researching = Some(TechId::Gunpowder);
        player.research_queue = vec![TechId::Gunpowder, TechId::Metallurgy];
        player.science_stored = 123;
        assert_eq!(migrate(&mut state).len(), 1);
        let player = state.player(id);
        assert!(player.techs.contains(&TechId::Gunpowder));
        assert_eq!(player.researching, Some(TechId::Metallurgy));
        assert_eq!(player.research_queue, vec![TechId::Metallurgy]);
        assert_eq!(player.science_stored, 123);
        assert!(migrate(&mut state).is_empty());
    }

    #[test]
    fn hold_biel_migration_restores_indias_coal_sight() {
        let file = ScenarioFile::parse(include_str!("../../../scenarios/lab/swiss-hold-biel.json"))
            .expect("Hold Biel card");
        let mut state = file.stored_state().expect("Hold Biel state");
        let india = state
            .players
            .iter()
            .find(|p| p.faction == Faction::India)
            .unwrap()
            .id;
        state
            .player_mut(india)
            .techs
            .remove(&TechId::Industrialization);
        assert!(state.player(india).techs.contains(&TechId::Metallurgy));
        let coal = state
            .map
            .tiles
            .iter()
            .position(|tile| tile.resource == Some(Resource::Coal))
            .expect("Coal on the map");
        assert_ne!(
            resources::visible_resource(&state.map.tiles[coal], true, &state.player(india).techs),
            Some(Resource::Coal)
        );
        assert!(
            migrate(&mut state)
                .iter()
                .any(|edit| edit.contains("India: Industrialization"))
        );
        assert_eq!(
            resources::visible_resource(&state.map.tiles[coal], true, &state.player(india).techs),
            Some(Resource::Coal)
        );
        assert_eq!(state.player(india).researching, Some(TechId::Railways));
        assert_eq!(state.player(india).science_stored, 191);
    }
}
