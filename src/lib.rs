use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Species
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Species {
    Grass,
    Bush,
    Tree,
    Rabbit,
    Fox,
    Wolf,
    Eagle,
    Fish,
    CrystalBug,
    Firefly,
}

impl Species {
    /// Trophic level: 0 = producer, 1 = primary consumer, 2 = predator, 3 = apex.
    pub fn trophic_level(&self) -> u32 {
        match self {
            Self::Grass | Self::Bush | Self::Tree => 0,
            Self::Rabbit | Self::CrystalBug | Self::Firefly | Self::Fish => 1,
            Self::Fox | Self::Eagle => 2,
            Self::Wolf => 3,
        }
    }

    /// Energy cost per tick — higher trophic levels pay more.
    pub fn energy_cost(&self) -> f64 {
        match self {
            Self::Grass => 0.005,
            Self::Bush => 0.005,
            Self::Tree => 0.005,
            Self::Rabbit => 0.02,
            Self::CrystalBug => 0.02,
            Self::Firefly => 0.02,
            Self::Fish => 0.02,
            Self::Fox => 0.04,
            Self::Eagle => 0.04,
            Self::Wolf => 0.06,
        }
    }

    /// Energy threshold above which an organism can reproduce.
    pub fn reproduction_threshold(&self) -> f64 {
        match self {
            Self::Grass => 1.5,
            Self::Bush => 1.5,
            Self::Tree => 2.0,
            Self::Rabbit => 2.5,
            Self::CrystalBug => 2.5,
            Self::Firefly => 2.5,
            Self::Fish => 2.5,
            Self::Fox => 3.0,
            Self::Eagle => 3.0,
            Self::Wolf => 4.0,
        }
    }

    /// All variants.
    pub fn all() -> &'static [Species] {
        &[
            Species::Grass,
            Species::Bush,
            Species::Tree,
            Species::Rabbit,
            Species::Fox,
            Species::Wolf,
            Species::Eagle,
            Species::Fish,
            Species::CrystalBug,
            Species::Firefly,
        ]
    }
}

// ---------------------------------------------------------------------------
// Organism
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organism {
    pub id: String,
    pub species: Species,
    pub energy: f64,
    pub age: u64,
    pub position: (usize, usize),
}

impl Organism {
    /// Advance one tick: age increases, energy decreases by species cost + base 0.01.
    /// Returns `false` if the organism is dead (energy ≤ 0).
    pub fn tick(&mut self) -> bool {
        self.age += 1;
        self.energy -= 0.01 + self.species.energy_cost();
        self.energy > 0.0
    }

    /// Create a child organism at a nearby position, transferring half the parent's energy.
    pub fn spawn_child(&self, child_id: String, new_position: (usize, usize)) -> Organism {
        let child_energy = self.energy / 2.0;
        Organism {
            id: child_id,
            species: self.species.clone(),
            energy: child_energy,
            age: 0,
            position: new_position,
        }
    }
}

// ---------------------------------------------------------------------------
// FoodWeb
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodWeb {
    pub predator_prey: Vec<(Species, Species)>,
}

impl FoodWeb {
    /// Build the default Lau food web.
    pub fn default_web() -> Self {
        use Species::*;
        Self {
            predator_prey: vec![
                (Rabbit, Grass),
                (Fox, Rabbit),
                (Wolf, Fox),
                (CrystalBug, Bush),
                (Eagle, CrystalBug),
                (Firefly, Tree),
                (Eagle, Fish),
                // Wolf also eats rabbits
                (Wolf, Rabbit),
            ],
        }
    }

    pub fn can_eat(&self, predator: &Species, prey: &Species) -> bool {
        self.predator_prey
            .iter()
            .any(|(p, q)| p == predator && q == prey)
    }

    pub fn prey_of(&self, predator: &Species) -> Vec<Species> {
        self.predator_prey
            .iter()
            .filter(|(p, _)| p == predator)
            .map(|(_, q)| q.clone())
            .collect()
    }

    pub fn predators_of(&self, prey: &Species) -> Vec<Species> {
        self.predator_prey
            .iter()
            .filter(|(_, q)| q == prey)
            .map(|(p, _)| p.clone())
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Population
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Population {
    pub species: Species,
    pub organisms: Vec<Organism>,
    pub carrying_capacity: usize,
}

impl Population {
    pub fn new(species: Species, carrying_capacity: usize) -> Self {
        Self {
            species,
            organisms: Vec::new(),
            carrying_capacity,
        }
    }

    pub fn size(&self) -> usize {
        self.organisms.len()
    }

    pub fn add(&mut self, organism: Organism) {
        if self.organisms.len() < self.carrying_capacity {
            self.organisms.push(organism);
        }
    }

    /// Remove dead organisms (energy ≤ 0). Returns count removed.
    pub fn remove_dead(&mut self) -> usize {
        let before = self.organisms.len();
        self.organisms.retain(|o| o.energy > 0.0);
        before - self.organisms.len()
    }

    /// Organisms above reproduction threshold produce offspring.
    /// Parent's energy is halved; child gets the other half. Returns new offspring.
    pub fn reproduce(&mut self, tick: u64) -> Vec<Organism> {
        let mut offspring = Vec::new();

        // Identify which organisms will reproduce
        let repro_indices: Vec<usize> = self
            .organisms
            .iter()
            .enumerate()
            .filter(|(_, org)| org.energy >= org.species.reproduction_threshold())
            .map(|(i, _)| i)
            .collect();

        for idx in repro_indices {
            let org = &mut self.organisms[idx];
            let child_energy = org.energy / 2.0;
            org.energy -= child_energy;
            let child_id = format!("{}-child-{}", org.id, tick);
            let dx = (org.position.0 as i64 + 1).max(0) as usize;
            let dy = org.position.1;
            let child = Organism {
                id: child_id,
                species: org.species.clone(),
                energy: child_energy,
                age: 0,
                position: (dx, dy),
            };
            offspring.push(child);
        }

        // Add children that fit within carrying capacity
        let mut accepted = Vec::new();
        for child in offspring {
            if self.organisms.len() < self.carrying_capacity {
                self.organisms.push(child.clone());
                accepted.push(child);
            }
        }

        accepted
    }
}

// ---------------------------------------------------------------------------
// Ecosystem
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ecosystem {
    pub populations: HashMap<Species, Population>,
    pub food_web: FoodWeb,
    pub total_energy: f64,
    pub tick_count: u64,
}

impl Ecosystem {
    /// Create a new ecosystem with the default food web and the given initial energy budget.
    pub fn new() -> Self {
        Self {
            populations: HashMap::new(),
            food_web: FoodWeb::default_web(),
            total_energy: 0.0,
            tick_count: 0,
        }
    }

    /// Seed a population into the ecosystem.
    pub fn seed(&mut self, pop: Population) {
        let energy: f64 = pop.organisms.iter().map(|o| o.energy).sum();
        self.total_energy += energy;
        self.populations.insert(pop.species.clone(), pop);
    }

    /// Total number of living organisms across all populations.
    pub fn total_population(&self) -> usize {
        self.populations.values().map(|p| p.size()).sum()
    }

    /// Shannon entropy of species populations — a biodiversity index.
    pub fn biodiversity(&self) -> f64 {
        let total = self.total_population() as f64;
        if total == 0.0 {
            return 0.0;
        }
        let mut entropy = 0.0;
        for pop in self.populations.values() {
            if pop.size() == 0 {
                continue;
            }
            let p = pop.size() as f64 / total;
            entropy -= p * p.log2();
        }
        entropy
    }

    /// Conservation error: absolute deviation of current total energy from initial.
    pub fn conservation_error(&self) -> f64 {
        let current: f64 = self
            .populations
            .values()
            .flat_map(|p| p.organisms.iter())
            .map(|o| o.energy)
            .sum();
        (current - self.total_energy).abs()
    }

    /// Is the ecosystem healthy? Biodiversity > 0.5 and every trophic level 0-3 has organisms.
    pub fn is_healthy(&self) -> bool {
        if self.biodiversity() <= 0.5 {
            return false;
        }
        for level in 0..=3u32 {
            let has = self.populations.values().any(|p| {
                p.size() > 0 && p.species.trophic_level() == level
            });
            if !has {
                return false;
            }
        }
        true
    }

    /// Advance the ecosystem by one tick:
    /// 1. All organisms metabolize.
    /// 2. Predators hunt prey (gain energy, prey lose energy).
    /// 3. Reproduction.
    /// 4. Remove dead.
    pub fn tick(&mut self) {
        self.tick_count += 1;

        // 1. Metabolize
        for pop in self.populations.values_mut() {
            for org in &mut pop.organisms {
                org.tick();
            }
        }

        // 2. Predation
        // For each predator-prey edge, predators gain energy from prey.
        // Simplified: each predator consumes a share of available prey energy.
        let edges = self.food_web.predator_prey.clone();
        let mut energy_transfers: HashMap<Species, f64> = HashMap::new();

        for (predator, prey) in &edges {
            let pred_count = self
                .populations
                .get(predator)
                .map(|p| p.size())
                .unwrap_or(0);
            if pred_count == 0 {
                continue;
            }

            let prey_pop = self.populations.get(prey);
            let total_prey_energy: f64 = prey_pop
                .map(|p| p.organisms.iter().map(|o| o.energy).sum())
                .unwrap_or(0.0);

            if total_prey_energy <= 0.0 {
                continue;
            }

            // Each predator consumes 0.1 energy from prey pool
            let consumption = (pred_count as f64 * 0.1).min(total_prey_energy * 0.5);
            *energy_transfers.entry(predator.clone()).or_default() += consumption;

            // Drain from prey
            if let Some(prey_pop) = self.populations.get_mut(prey) {
                let drain_per = consumption / prey_pop.size().max(1) as f64;
                for org in &mut prey_pop.organisms {
                    org.energy -= drain_per.min(org.energy);
                }
            }
        }

        // Apply energy gains
        for (species, gain) in energy_transfers {
            if let Some(pop) = self.populations.get_mut(&species) {
                let per = gain / pop.size().max(1) as f64;
                for org in &mut pop.organisms {
                    org.energy += per;
                }
            }
        }

        // 3. Reproduction
        for species in Species::all() {
            if let Some(pop) = self.populations.get_mut(species) {
                pop.reproduce(self.tick_count);
            }
        }

        // 4. Remove dead
        for pop in self.populations.values_mut() {
            pop.remove_dead();
        }
    }
}

impl Default for Ecosystem {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_organism(id: &str, species: Species, energy: f64) -> Organism {
        Organism {
            id: id.to_string(),
            species: species.clone(),
            energy,
            age: 0,
            position: (0, 0),
        }
    }

    fn seeded_ecosystem() -> Ecosystem {
        let mut eco = Ecosystem::new();

        let mut grass_pop = Population::new(Species::Grass, 100);
        for i in 0..20 {
            grass_pop.add(make_organism(&format!("grass-{i}"), Species::Grass, 2.0));
        }

        let mut rabbit_pop = Population::new(Species::Rabbit, 50);
        for i in 0..10 {
            rabbit_pop.add(make_organism(&format!("rabbit-{i}"), Species::Rabbit, 3.0));
        }

        let mut fox_pop = Population::new(Species::Fox, 20);
        for i in 0..5 {
            fox_pop.add(make_organism(&format!("fox-{i}"), Species::Fox, 4.0));
        }

        let mut wolf_pop = Population::new(Species::Wolf, 10);
        for i in 0..2 {
            wolf_pop.add(make_organism(&format!("wolf-{i}"), Species::Wolf, 5.0));
        }

        let mut bush_pop = Population::new(Species::Bush, 50);
        for i in 0..10 {
            bush_pop.add(make_organism(&format!("bush-{i}"), Species::Bush, 2.0));
        }

        let mut crystal_pop = Population::new(Species::CrystalBug, 30);
        for i in 0..5 {
            crystal_pop.add(make_organism(&format!("crystal-{i}"), Species::CrystalBug, 3.0));
        }

        let mut eagle_pop = Population::new(Species::Eagle, 10);
        for i in 0..3 {
            eagle_pop.add(make_organism(&format!("eagle-{i}"), Species::Eagle, 4.0));
        }

        let mut fish_pop = Population::new(Species::Fish, 30);
        for i in 0..8 {
            fish_pop.add(make_organism(&format!("fish-{i}"), Species::Fish, 2.5));
        }

        let mut tree_pop = Population::new(Species::Tree, 50);
        for i in 0..10 {
            tree_pop.add(make_organism(&format!("tree-{i}"), Species::Tree, 2.0));
        }

        let mut firefly_pop = Population::new(Species::Firefly, 30);
        for i in 0..5 {
            firefly_pop.add(make_organism(&format!("firefly-{i}"), Species::Firefly, 3.0));
        }

        eco.seed(grass_pop);
        eco.seed(rabbit_pop);
        eco.seed(fox_pop);
        eco.seed(wolf_pop);
        eco.seed(bush_pop);
        eco.seed(crystal_pop);
        eco.seed(eagle_pop);
        eco.seed(fish_pop);
        eco.seed(tree_pop);
        eco.seed(firefly_pop);

        eco
    }

    // -- Species tests --

    #[test]
    fn trophic_levels() {
        assert_eq!(Species::Grass.trophic_level(), 0);
        assert_eq!(Species::Rabbit.trophic_level(), 1);
        assert_eq!(Species::Fox.trophic_level(), 2);
        assert_eq!(Species::Wolf.trophic_level(), 3);
        assert_eq!(Species::Eagle.trophic_level(), 2);
    }

    #[test]
    fn energy_cost_increases_with_trophic_level() {
        assert!(Species::Grass.energy_cost() < Species::Rabbit.energy_cost());
        assert!(Species::Rabbit.energy_cost() < Species::Fox.energy_cost());
        assert!(Species::Fox.energy_cost() < Species::Wolf.energy_cost());
    }

    #[test]
    fn reproduction_thresholds() {
        assert!(Species::Grass.reproduction_threshold() < Species::Wolf.reproduction_threshold());
    }

    #[test]
    fn species_all_has_10_variants() {
        assert_eq!(Species::all().len(), 10);
    }

    // -- Organism tests --

    #[test]
    fn organism_tick_ages_and_metabolizes() {
        let mut org = make_organism("test", Species::Grass, 1.0);
        let cost = 0.01 + Species::Grass.energy_cost();
        assert!(org.tick());
        assert_eq!(org.age, 1);
        assert!((org.energy - (1.0 - cost)).abs() < 1e-10);
    }

    #[test]
    fn organism_dies_when_energy_depleted() {
        let mut org = make_organism("test", Species::Grass, 0.001);
        assert!(!org.tick());
    }

    #[test]
    fn spawn_child_splits_energy() {
        let org = make_organism("parent", Species::Rabbit, 4.0);
        let child = org.spawn_child("child".into(), (1, 0));
        assert_eq!(child.energy, 2.0);
        assert_eq!(child.age, 0);
        assert_eq!(child.position, (1, 0));
    }

    // -- FoodWeb tests --

    #[test]
    fn can_eat_default_web() {
        let web = FoodWeb::default_web();
        assert!(web.can_eat(&Species::Rabbit, &Species::Grass));
        assert!(web.can_eat(&Species::Fox, &Species::Rabbit));
        assert!(web.can_eat(&Species::Wolf, &Species::Fox));
        assert!(web.can_eat(&Species::Eagle, &Species::CrystalBug));
        assert!(web.can_eat(&Species::Eagle, &Species::Fish));
        assert!(web.can_eat(&Species::Firefly, &Species::Tree));
        assert!(!web.can_eat(&Species::Grass, &Species::Rabbit));
        assert!(!web.can_eat(&Species::Rabbit, &Species::Wolf));
    }

    #[test]
    fn prey_of_returns_correct_prey() {
        let web = FoodWeb::default_web();
        let prey = web.prey_of(&Species::Eagle);
        assert!(prey.contains(&Species::CrystalBug));
        assert!(prey.contains(&Species::Fish));
        assert_eq!(prey.len(), 2);
    }

    #[test]
    fn predators_of_returns_correct_predators() {
        let web = FoodWeb::default_web();
        let preds = web.predators_of(&Species::Rabbit);
        assert!(preds.contains(&Species::Fox));
        assert!(preds.contains(&Species::Wolf));
    }

    #[test]
    fn wolf_eats_rabbit_and_fox() {
        let web = FoodWeb::default_web();
        assert!(web.can_eat(&Species::Wolf, &Species::Rabbit));
        assert!(web.can_eat(&Species::Wolf, &Species::Fox));
    }

    // -- Population tests --

    #[test]
    fn population_add_and_size() {
        let mut pop = Population::new(Species::Grass, 5);
        assert_eq!(pop.size(), 0);
        pop.add(make_organism("g1", Species::Grass, 1.0));
        assert_eq!(pop.size(), 1);
    }

    #[test]
    fn population_respects_carrying_capacity() {
        let mut pop = Population::new(Species::Grass, 2);
        pop.add(make_organism("g1", Species::Grass, 1.0));
        pop.add(make_organism("g2", Species::Grass, 1.0));
        pop.add(make_organism("g3", Species::Grass, 1.0)); // should be rejected
        assert_eq!(pop.size(), 2);
    }

    #[test]
    fn population_remove_dead() {
        let mut pop = Population::new(Species::Grass, 10);
        pop.add(make_organism("g1", Species::Grass, 1.0));
        pop.add(make_organism("g2", Species::Grass, -1.0));
        pop.add(make_organism("g3", Species::Grass, 0.0));
        assert_eq!(pop.remove_dead(), 2);
        assert_eq!(pop.size(), 1);
    }

    #[test]
    fn population_reproduce() {
        let mut pop = Population::new(Species::Grass, 100);
        pop.add(make_organism("g1", Species::Grass, 5.0));
        let children = pop.reproduce(1);
        assert_eq!(children.len(), 1);
        assert!(pop.size() > 1);
    }

    #[test]
    fn population_reproduce_respects_capacity() {
        let mut pop = Population::new(Species::Grass, 1);
        pop.add(make_organism("g1", Species::Grass, 5.0));
        let children = pop.reproduce(1);
        assert_eq!(children.len(), 0); // already at capacity
    }

    // -- Ecosystem tests --

    #[test]
    fn ecosystem_total_population() {
        let eco = seeded_ecosystem();
        assert_eq!(eco.total_population(), 78);
    }

    #[test]
    fn ecosystem_biodiversity_positive() {
        let eco = seeded_ecosystem();
        let bio = eco.biodiversity();
        assert!(bio > 0.0, "biodiversity should be positive: {bio}");
    }

    #[test]
    fn ecosystem_is_healthy_with_all_levels() {
        let eco = seeded_ecosystem();
        assert!(eco.is_healthy(), "seeded ecosystem should be healthy");
    }

    #[test]
    fn ecosystem_not_healthy_missing_trophic_level() {
        let mut eco = Ecosystem::new();
        // Only producers — no consumers or predators
        let mut pop = Population::new(Species::Grass, 100);
        pop.add(make_organism("g1", Species::Grass, 2.0));
        eco.seed(pop);
        assert!(!eco.is_healthy());
    }

    #[test]
    fn ecosystem_tick_advances() {
        let mut eco = seeded_ecosystem();
        assert_eq!(eco.tick_count, 0);
        eco.tick();
        assert_eq!(eco.tick_count, 1);
    }

    #[test]
    fn ecosystem_tick_metabolism_drains_energy() {
        // Use a simple setup with no predators to avoid predation/reproduction confounds
        let mut eco = Ecosystem::new();
        let mut pop = Population::new(Species::Grass, 100);
        pop.add(make_organism("g1", Species::Grass, 5.0));
        eco.seed(pop);
        let initial_energy: f64 = eco
            .populations
            .values()
            .flat_map(|p| p.organisms.iter())
            .map(|o| o.energy)
            .sum();
        eco.tick();
        let after_energy: f64 = eco
            .populations
            .values()
            .flat_map(|p| p.organisms.iter())
            .map(|o| o.energy)
            .sum();
        // Energy decreases due to metabolism
        assert!(after_energy < initial_energy, "after={after_energy} initial={initial_energy}");
    }

    #[test]
    fn ecosystem_conservation_error() {
        let eco = seeded_ecosystem();
        // Before any ticks, error should be 0
        assert!((eco.conservation_error()).abs() < 1e-10);
    }

    #[test]
    fn ecosystem_tick_multiple_times() {
        let mut eco = seeded_ecosystem();
        for _ in 0..10 {
            eco.tick();
        }
        assert_eq!(eco.tick_count, 10);
        assert!(eco.total_population() > 0);
    }

    #[test]
    fn ecosystem_default() {
        let eco = Ecosystem::default();
        assert_eq!(eco.tick_count, 0);
        assert_eq!(eco.total_population(), 0);
    }

    #[test]
    fn biodiversity_empty_ecosystem() {
        let eco = Ecosystem::new();
        assert_eq!(eco.biodiversity(), 0.0);
    }

    // -- Serde tests --

    #[test]
    fn serde_species_roundtrip() {
        let species = Species::Wolf;
        let json = serde_json::to_string(&species).unwrap();
        let back: Species = serde_json::from_str(&json).unwrap();
        assert_eq!(species, back);
    }

    #[test]
    fn serde_organism_roundtrip() {
        let org = make_organism("test", Species::Fox, 3.14);
        let json = serde_json::to_string(&org).unwrap();
        let back: Organism = serde_json::from_str(&json).unwrap();
        assert_eq!(org.id, back.id);
        assert!((org.energy - back.energy).abs() < 1e-10);
    }

    #[test]
    fn serde_ecosystem_roundtrip() {
        let eco = seeded_ecosystem();
        let json = serde_json::to_string(&eco).unwrap();
        let back: Ecosystem = serde_json::from_str(&json).unwrap();
        assert_eq!(eco.total_population(), back.total_population());
        assert_eq!(eco.tick_count, back.tick_count);
    }
}
