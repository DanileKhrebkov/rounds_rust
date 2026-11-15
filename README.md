# ROUNDS White-Room Prototype — Bevy 0.19 Step-by-Step (From Step 1)

**Bevy 0.19 API constraints applied throughout:** `Resource` is now a subtrait of `Component` — do not derive both on the same type . `Time::delta_seconds()` is renamed to `delta_secs()` . `Camera2dBundle` is gone — use `Camera2d` unit struct . Prefer `Sprite::from_color(color, size)` for prototyping . `rand` is on 0.10 in the 0.19 ecosystem — `thread_rng` becomes `rng()` and `RngCore` trait becomes `Rng` .

---

## Step 1 — Project Setup & Crate Structure (Bevy 0.19)

**Goal:** Compiling app with correct 0.19 dependencies and module layout.

**Cargo.toml:**
```toml
[package]
name = "rounds_white"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = "0.19"
rand = "0.10"
```

> **0.19 note:** No `bevy_rapier2d` or `avian2d` in this white-room plan. We implement circle-circle collision manually to avoid dependency lock-in. If you later add physics, verify the crate has a 0.19-compatible release — some ecosystem crates lag behind .

**Module layout:**
```
src/
  main.rs
  components/
    mod.rs
    player.rs
    projectile.rs
    cards.rs
  resources/
    mod.rs
    round_state.rs
    config.rs
  systems/
    mod.rs
    input.rs
    movement.rs
    combat.rs
    draft.rs
    lifecycle.rs
```

**main.rs skeleton:**
```rust
use bevy::prelude::*;

mod components;
mod resources;
mod systems;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<resources::Config>()
        .init_resource::<resources::RoundState>()
        .init_resource::<resources::ScoreBoard>()
        .init_resource::<resources::DraftState>()
        .init_resource::<resources::CardPool>()
        .add_systems(Startup, systems::lifecycle::setup)
        .add_systems(Update, (
            systems::input::input_system,
            systems::input::jump_system,
            systems::movement::gravity_system,
            systems::combat::shoot_system,
            systems::combat::projectile_move_system,
            systems::combat::projectile_lifetime_system,
            systems::combat::collision_system,
            systems::lifecycle::death_system,
        ))
        .run();
}
```

**Acceptance:** `cargo run` opens an empty window.

---

## Step 2 — Spawn Two Players (Bevy 0.19 Sprites & Camera)

**Goal:** Two circles with full stat components, plus a 2D camera.

**Components (`components/player.rs`):**
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerTag { pub index: usize }

#[derive(Component)]
pub struct Health { pub current: f32, pub max: f32 }

#[derive(Component)]
pub struct Velocity { pub x: f32, pub y: f32 }

#[derive(Component)]
pub struct OnGround(pub bool);

#[derive(Component)]
pub struct MoveSpeed(pub f32);

#[derive(Component)]
pub struct BlockState { pub active: bool }

#[derive(Component)]
pub struct Ammo { pub current: u32, pub max: u32 }

#[derive(Component)]
pub struct Facing(pub f32); // -1.0 left, 1.0 right
```

**Config resource (`resources/config.rs`):**
```rust
use bevy::prelude::*;

#[derive(Resource)] // Do NOT also derive Component in 0.19
pub struct Config {
    pub gravity: f32,
    pub jump_impulse: f32,
    pub ground_y: f32,
    pub player_radius: f32,
    pub projectile_speed: f32,
    pub projectile_damage: f32,
    pub projectile_radius: f32,
    pub projectile_lifetime: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gravity: -30.0,
            jump_impulse: 12.0,
            ground_y: -200.0,
            player_radius: 20.0,
            projectile_speed: 15.0,
            projectile_damage: 10.0,
            projectile_radius: 5.0,
            projectile_lifetime: 2.0,
        }
    }
}
```

**Setup system (`systems/lifecycle.rs`):**
```rust
use bevy::prelude::*;
use crate::components::player::*;

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2d); // 0.19 idiom: no Camera2dBundle 

    let colors = [Color::srgb(0.2, 0.4, 1.0), Color::srgb(1.0, 0.2, 0.2)];

    for (i, x) in [(-200.0_f32, 0_usize), (200.0, 1)].iter() {
        let idx = x.1;
        commands.spawn((
            Sprite::from_color(colors[idx], Vec2::splat(40.0)), // 0.19 idiom 
            Transform::from_xyz(x.0, 0.0, 0.0),
            PlayerTag { index: idx },
            Health { current: 100.0, max: 100.0 },
            Velocity { x: 0.0, y: 0.0 },
            OnGround(false),
            MoveSpeed(8.0),
            BlockState { active: false },
            Ammo { current: 6, max: 6 },
            Facing(if idx == 0 { 1.0 } else { -1.0 }),
        ));
    }
}
```

**Acceptance:** Blue circle left, red circle right, both visible.

---

## Step 3 — Gravity & Ground Collision (Bevy 0.19 Time API)

**Goal:** Players fall to `ground_y` and stop.

**Critical 0.19 change:** Use `time.delta_secs()`, **not** `delta_seconds()` .

**System (`systems/movement.rs`):**
```rust
use bevy::prelude::*;
use crate::components::player::*;
use crate::resources::Config;

pub fn gravity_system(
    time: Res<Time>,
    cfg: Res<Config>,
    mut q: Query<(&mut Velocity, &mut Transform, &mut OnGround), With<PlayerTag>>,
) {
    let dt = time.delta_secs(); // 0.19: delta_seconds() removed

    for (mut vel, mut tf, mut og) in &mut q {
        vel.y += cfg.gravity * dt;
        tf.translation.y += vel.y * dt;
        tf.translation.x += vel.x * dt;

        if tf.translation.y <= cfg.ground_y {
            tf.translation.y = cfg.ground_y;
            vel.y = 0.0;
            og.0 = true;
        } else {
            og.0 = false;
        }
    }
}
```

**Data sources:**

| Data | Source | Bevy 0.19 note |
|---|---|---|
| Delta time | `Res<Time>` → `time.delta_secs()` | Renamed from `delta_seconds`  |
| Gravity config | `Res<Config>` | Resource is subtrait of Component — derive only `Resource` |
| Player state | `Query<(&mut Velocity, &mut Transform, &mut OnGround)>` | Unchanged |

**Acceptance:** Circles drop and rest on ground line.

---

## Step 4 — Horizontal Input & Movement

**Goal:** `A`/`D` moves player 0.

**System (`systems/input.rs`):**
```rust
use bevy::prelude::*;
use crate::components::player::*;

pub fn input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<(&mut Velocity, &PlayerTag, &MoveSpeed)>,
) {
    for (mut vel, tag, speed) in &mut q {
        if tag.index != 0 { continue; }
        let mut dir = 0.0;
        if keys.pressed(KeyCode::KeyA) { dir -= 1.0; }
        if keys.pressed(KeyCode::KeyD) { dir += 1.0; }
        vel.x = dir * speed.0;
    }
}
```

**Data sources:**

| Data | Source |
|---|---|
| Key state | `Res<ButtonInput<KeyCode>>` — `pressed`/`just_pressed` valid in 0.19  |
| Player movement | `Query<(&mut Velocity, &PlayerTag, &MoveSpeed)>` |

**Acceptance:** Blue circle moves left/right. No inertia yet.

---

## Step 5 — Jump (Grounded Check)

**Goal:** `Space` jumps only when `OnGround`.

**System (`systems/input.rs`):**
```rust
pub fn jump_system(
    keys: Res<ButtonInput<KeyCode>>,
    cfg: Res<Config>,
    mut q: Query<(&mut Velocity, &OnGround, &PlayerTag)>,
) {
    if !keys.just_pressed(KeyCode::Space) { return; }
    for (mut vel, og, tag) in &mut q {
        if tag.index == 0 && og.0 {
            vel.y = cfg.jump_impulse;
        }
    }
}
```

**Acceptance:** Single jump per landing. No double-jump.

---

## Step 6 — Facing Direction

**Goal:** Flip `Facing` based on movement direction.

**System (`systems/input.rs`):**
```rust
pub fn facing_system(
    mut q: Query<(&Velocity, &mut Facing, &PlayerTag)>,
) {
    for (vel, mut facing, tag) in &mut q {
        if tag.index != 0 { continue; }
        if vel.x > 0.1 { facing.0 = 1.0; }
        else if vel.x < -0.1 { facing.0 = -1.0; }
    }
}
```

> **White-room simplification:** Later replace with auto-aim toward opponent (ROUNDS' actual behavior) or mouse aim.

**Acceptance:** `Facing.0` flips with movement.

---

## Step 7 — Shooting (Spawn Projectile via Commands)

**Goal:** `J` spawns a bullet from player 0 toward `Facing`.

**Components (`components/projectile.rs`):**
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct ProjectileTag;

#[derive(Component)]
pub struct ProjectileOwner(pub Entity);

#[derive(Component)]
pub struct Damage(pub f32);

#[derive(Component)]
pub struct Lifetime(pub Timer);
```

**System (`systems/combat.rs`):**
```rust
use bevy::prelude::*;
use crate::components::player::*;
use crate::components::projectile::*;
use crate::resources::Config;

pub fn shoot_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    cfg: Res<Config>,
    mut q: Query<(Entity, &Transform, &Facing, &mut Ammo, &PlayerTag)>,
) {
    if !keys.just_pressed(KeyCode::KeyJ) { return; }

    for (entity, tf, facing, mut ammo, tag) in &mut q {
        if tag.index != 0 { continue; }
        if ammo.current == 0 { return; }

        ammo.current -= 1;
        let dir = Vec2::new(facing.0, 0.0);
        let spawn = tf.translation.truncate() + dir * 30.0;

        commands.spawn((
            Sprite::from_color(Color::srgb(1.0, 0.9, 0.2), Vec2::splat(cfg.projectile_radius * 2.0)),
            Transform::from_xyz(spawn.x, spawn.y, 0.0),
            ProjectileTag,
            ProjectileOwner(entity),
            Damage(cfg.projectile_damage),
            Velocity { x: dir.x * cfg.projectile_speed, y: 0.0 },
            Lifetime(Timer::from_seconds(cfg.projectile_lifetime, TimerMode::Once)),
        ));
    }
}
```

**Data sources:**

| Data | Source | 0.19 note |
|---|---|---|
| Input | `Res<ButtonInput<KeyCode>>` | Valid  |
| Shooter state | `Query<(Entity, &Transform, &Facing, &mut Ammo)>` | Unchanged |
| Spawning | `Commands` | Deferred; applies at schedule end  |

**Acceptance:** `J` spawns yellow square flying forward. Ammo decrements.

---

## Step 8 — Projectile Movement & Lifetime

**Goal:** Bullets move and despawn after timer.

**Systems (`systems/combat.rs`):**
```rust
pub fn projectile_move_system(
    time: Res<Time>,
    mut q: Query<(&Velocity, &mut Transform), With<ProjectileTag>>,
) {
    let dt = time.delta_secs();
    for (vel, mut tf) in &mut q {
        tf.translation.x += vel.x * dt;
        tf.translation.y += vel.y * dt;
    }
}

pub fn projectile_lifetime_system(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Lifetime), With<ProjectileTag>>,
) {
    for (entity, mut life) in &mut q {
        life.0.tick(time.delta());
        if life.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}
```

**0.19 note:** `despawn()` still valid. Use `despawn_recursive()` if projectiles ever gain children .

**Acceptance:** Bullets travel and vanish after 2s.

---

## Step 9 — Hit Detection & Damage

**Goal:** Bullet hitting enemy reduces health; blocked bullets deal zero damage.

**System (`systems/combat.rs`):**
```rust
use bevy::prelude::*;
use crate::components::player::*;
use crate::components::projectile::*;
use crate::resources::Config;

pub fn collision_system(
    mut commands: Commands,
    cfg: Res<Config>,
    bullets: Query<(Entity, &Transform, &Damage, &ProjectileOwner), With<ProjectileTag>>,
    mut players: Query<(Entity, &Transform, &mut Health, &BlockState), With<PlayerTag>>,
) {
    for (bullet, b_tf, dmg, owner) in &bullets {
        let b_pos = b_tf.translation.truncate();

        for (p_ent, p_tf, mut hp, block) in &mut players {
            if p_ent == owner.0 { continue; }
            let p_pos = p_tf.translation.truncate();
            let dist = b_pos.distance(p_pos);

            if dist < cfg.player_radius + cfg.projectile_radius {
                if !block.active {
                    hp.current -= dmg.0;
                }
                commands.entity(bullet).despawn();
                break;
            }
        }
    }
}
```

**Acceptance:** Shooting red circle reduces its `Health.current`. Holding `K` (see Step 10) prevents damage.

---

## Step 10 — Block Mechanic

**Goal:** Hold `K` to block.

**System (`systems/input.rs`):**
```rust
pub fn block_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<(&mut BlockState, &PlayerTag)>,
) {
    for (mut block, tag) in &mut q {
        if tag.index != 0 { continue; }
        block.active = keys.pressed(KeyCode::KeyK);
    }
}
```

Add `systems::input::block_system` to `Update`.

**Acceptance:** `BlockState.active` toggles with `K`. Blocked bullets despawn without damage.

---

## Step 11 — Death Detection & Round Phase Transition

**Goal:** When a player hits 0 HP, enter `Drafting` and record loser.

**Resources (`resources/round_state.rs`):**
```rust
use bevy::prelude::*;

#[derive(Resource, PartialEq, Eq, Clone, Copy)]
pub enum RoundPhase { Combat, Drafting, Cleanup, MatchOver }

#[derive(Resource)]
pub struct RoundState {
    pub phase: RoundPhase,
    pub round_number: u32,
    pub loser: Option<usize>,
}

impl Default for RoundState {
    fn default() -> Self {
        Self { phase: RoundPhase::Combat, round_number: 1, loser: None }
    }
}

#[derive(Resource)]
pub struct ScoreBoard {
    pub wins: [u32; 2],
    pub needed: u32,
}

impl Default for ScoreBoard {
    fn default() -> Self {
        Self { wins: [0, 0], needed: 5 }
    }
}
```

**System (`systems/lifecycle.rs`):**
```rust
use bevy::prelude::*;
use crate::components::player::*;
use crate::resources::{RoundPhase, RoundState, ScoreBoard};

pub fn death_system(
    mut round: ResMut<RoundState>,
    mut score: ResMut<ScoreBoard>,
    players: Query<(&Health, &PlayerTag)>,
) {
    if round.phase != RoundPhase::Combat { return; }

    let mut dead: Vec<usize> = vec![];
    for (hp, tag) in &players {
        if hp.current <= 0.0 { dead.push(tag.index); }
    }

    if let Some(&loser) = dead.first() {
        round.loser = Some(loser);
        round.phase = RoundPhase::Drafting;
        let winner = 1 - loser;
        score.wins[winner] += 1;
        println!("Player {} lost round {}", loser, round.round_number);
    }
}
```

**Data sources:**

| Data | Source |
|---|---|
| Health | `Query<(&Health, &PlayerTag)>` |
| Phase / loser | `ResMut<RoundState>` |
| Score | `ResMut<ScoreBoard>` |

**Acceptance:** HP reaching 0 freezes combat and prints loser.

---

## Step 12 — Card Data Layer (Bevy 0.19 Resources)

**Goal:** Card catalog as resource data, no behavior yet.

**Components (`components/cards.rs`):**
```rust
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CardId { BigBullet, MoreAmmo, FastReload, ExtraHealth, Homing, Bounce }

#[derive(Clone, Copy, Debug)]
pub enum StatType { Damage, Ammo, Health }

#[derive(Clone, Copy, Debug)]
pub enum StatModifier {
    Add(StatType, f32),
    Mul(StatType, f32),
}

#[derive(Clone, Debug)]
pub struct CardDef {
    pub id: CardId,
    pub name: &'static str,
    pub modifiers: Vec<StatModifier>,
    pub behavior: Option<BehaviorTag>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviorTag { Homing, Bounce }

#[derive(Component, Default)]
pub struct CardInventory { pub cards: Vec<CardId> }

#[derive(Component)]
pub struct StatBlock {
    pub damage_mult: f32,
    pub ammo_add: i32,
    pub health_add: f32,
}

impl Default for StatBlock {
    fn default() -> Self {
        Self { damage_mult: 1.0, ammo_add: 0, health_add: 0.0 }
    }
}
```

**Resource (`resources/round_state.rs` or new `resources/cards.rs`):**
```rust
use bevy::prelude::*;
use crate::components::cards::*;

#[derive(Resource)]
pub struct CardPool { pub defs: Vec<CardDef> }

impl Default for CardPool {
    fn default() -> Self {
        Self {
            defs: vec![
                CardDef {
                    id: CardId::BigBullet,
                    name: "Big Bullet",
                    modifiers: vec![StatModifier::Mul(StatType::Damage, 1.5)],
                    behavior: None,
                },
                CardDef {
                    id: CardId::MoreAmmo,
                    name: "More Ammo",
                    modifiers: vec![StatModifier::Add(StatType::Ammo, 3.0)],
                    behavior: None,
                },
                CardDef {
                    id: CardId::FastReload,
                    name: "Fast Reload",
                    modifiers: vec![],
                    behavior: None,
                },
                CardDef {
                    id: CardId::ExtraHealth,
                    name: "Extra Health",
                    modifiers: vec![StatModifier::Add(StatType::Health, 25.0)],
                    behavior: None,
                },
                CardDef {
                    id: CardId::Homing,
                    name: "Homing",
                    modifiers: vec![],
                    behavior: Some(BehaviorTag::Homing),
                },
                CardDef {
                    id: CardId::Bounce,
                    name: "Bounce",
                    modifiers: vec![],
                    behavior: Some(BehaviorTag::Bounce),
                },
            ],
        }
    }
}
```

**0.19 note:** `CardPool` is a `Resource`. Do **not** derive `Component` on it. The resource-as-component change means you cannot treat `CardPool` as an entity component .

**Acceptance:** Compiles. No behavior changes yet.

---

## Step 13 — Draft Phase: Offer Cards

**Goal:** On entering `Drafting`, offer 3 random cards to the loser.

**Resource (`resources/round_state.rs`):**
```rust
#[derive(Resource, Default)]
pub struct DraftState {
    pub choices: Vec<CardId>,
    pub chosen: Option<CardId>,
}
```

**System (`systems/draft.rs`):**
```rust
use bevy::prelude::*;
use rand::seq::IndexedRandom; // rand 0.10 trait
use crate::components::cards::CardId;
use crate::resources::{RoundPhase, RoundState, DraftState, CardPool};

pub fn draft_offer_system(
    round: Res<RoundState>,
    mut draft: ResMut<DraftState>,
    pool: Res<CardPool>,
) {
    if round.phase != RoundPhase::Drafting || !draft.choices.is_empty() { return; }

    let mut rng = rand::rng(); // rand 0.10: thread_rng() → rng()
    let all_ids: Vec<CardId> = pool.defs.iter().map(|c| c.id).collect();

    draft.choices = all_ids
        .choose_multiple(&mut rng, 3)
        .cloned()
        .collect();

    println!("Draft choices: {:?}", draft.choices);
}
```

> **rand 0.10 note:** The 0.19 ecosystem bumped to `rand` 0.10. `thread_rng()` becomes `rng()`, and `RngCore` becomes `Rng` . If compilation fails, check your `Cargo.lock` — you may still be on 0.8 depending on transitive deps.

**Data sources:**

| Data | Source |
|---|---|
| Phase | `Res<RoundState>` |
| Draft choices | `ResMut<DraftState>` |
| Card catalog | `Res<CardPool>` |

**Acceptance:** After death, console prints 3 card IDs.

---

## Step 14 — Draft Selection & Application (Bevy 0.19 Input)

**Goal:** Player picks with `1`/`2`/`3`; card enters `CardInventory`; `StatBlock` recalculates.

**System 1: Selection (`systems/draft.rs`):**
```rust
use bevy::prelude::*;
use crate::components::cards::CardId;
use crate::resources::{RoundPhase, RoundState, DraftState};

pub fn draft_select_system(
    keys: Res<ButtonInput<KeyCode>>,
    round: Res<RoundState>,
    mut draft: ResMut<DraftState>,
) {
    if round.phase != RoundPhase::Drafting { return; }
    if draft.chosen.is_some() { return; }

    let pick = if keys.just_pressed(KeyCode::Digit1) { Some(0) }
        else if keys.just_pressed(KeyCode::Digit2) { Some(1) }
        else if keys.just_pressed(KeyCode::Digit3) { Some(2) }
        else { None };

    if let Some(idx) = pick {
        if let Some(&card) = draft.choices.get(idx) {
            draft.chosen = Some(card);
            println!("Player picked: {:?}", card);
        }
    }
}
```

**System 2: Application (`systems/draft.rs`):**
```rust
use bevy::prelude::*;
use crate::components::cards::*;
use crate::components::player::PlayerTag;
use crate::resources::{RoundPhase, RoundState, DraftState, CardPool};

pub fn apply_card_system(
    round: Res<RoundState>,
    mut draft: ResMut<DraftState>,
    pool: Res<CardPool>,
    mut players: Query<(&mut CardInventory, &PlayerTag)>,
) {
    if round.phase != RoundPhase::Drafting { return; }
    let Some(chosen) = draft.chosen.take() else { return; };
    let Some(loser_idx) = round.loser else { return; };

    for (mut inv, tag) in &mut players {
        if tag.index == loser_idx {
            inv.cards.push(chosen);
            if let Some(def) = pool.defs.iter().find(|d| d.id == chosen) {
                println!("Player {} gained: {}", loser_idx, def.name);
            }
        }
    }

    draft.choices.clear();
}
```

**System 3: Stat Recalculation (`systems/draft.rs`):**
```rust
use bevy::prelude::*;
use crate::components::cards::*;
use crate::components::player::PlayerTag;
use crate::resources::CardPool;

pub fn recalculate_stats_system(
    pool: Res<CardPool>,
    mut players: Query<(&CardInventory, &mut StatBlock), With<PlayerTag>>,
) {
    for (inv, mut stats) in &mut players {
        *stats = StatBlock::default(); // reset

        for card_id in &inv.cards {
            if let Some(def) = pool.defs.iter().find(|d| d.id == *card_id) {
                for modifier in &def.modifiers {
                    match modifier {
                        StatModifier::Mul(StatType::Damage, val) => stats.damage_mult *= val,
                        StatModifier::Add(StatType::Ammo, val) => stats.ammo_add += *val as i32,
                        StatModifier::Add(StatType::Health, val) => stats.health_add += val,
                        _ => {}
                    }
                }
            }
        }
    }
}
```

**Data sources:**

| Data | Source | 0.19 note |
|---|---|---|
| Selection input | `Res<ButtonInput<KeyCode>>` | `Digit1`/`Digit2`/`Digit3` valid  |
| Card choice | `ResMut<DraftState>` | Resource |
| Player inventory | `Query<(&mut CardInventory, &PlayerTag)>` | Component |
| Card catalog | `Res<CardPool>` | Resource |
| Stat aggregation | `Query<(&CardInventory, &mut StatBlock)>` | Component |

Add all three draft systems to `Update`.

**Acceptance:** Pressing `1`/`2`/`3` adds the card to the loser's inventory and prints the card name.

---

## Step 15 — Apply Stats to Combat & Reset Round

**Goal:** `StatBlock` actually modifies damage/ammo/health. Round resets to `Combat`.

**Modify `shoot_system`** to read `StatBlock`:

```rust
// Add to shoot_system query:
mut q: Query<(Entity, &Transform, &Facing, &mut Ammo, &StatBlock), With<PlayerTag>>,

// Inside loop, replace damage line:
let actual_damage = cfg.projectile_damage * stats.damage_mult;
// Use actual_damage when spawning Damage component
```

**Modify `setup`** to give players `CardInventory::default()` and `StatBlock::default()`.

**Reset system (`systems/lifecycle.rs`):**
```rust
use bevy::prelude::*;
use crate::components::player::*;
use crate::components::projectile::ProjectileTag;
use crate::resources::{RoundPhase, RoundState, DraftState};

pub fn round_reset_system(
    mut commands: Commands,
    mut round: ResMut<RoundState>,
    mut draft: ResMut<DraftState>,
    mut players: Query<(&mut Health, &mut Ammo, &mut Transform, &mut Velocity), With<PlayerTag>>,
    projectiles: Query<Entity, With<ProjectileTag>>,
) {
    if round.phase != RoundPhase::Drafting { return; }
    // Only reset after a card was chosen
    // (draft.chosen was taken; check if we just finished)
    // Simplest: reset when phase is Drafting AND no choices AND chosen is None
    if !draft.choices.is_empty() { return; }

    // Despawn all projectiles
    for entity in &projectiles {
        commands.entity(entity).despawn();
    }

    // Reset players
    for (mut hp, mut ammo, mut tf, mut vel) in &mut players {
        hp.current = hp.max;
        ammo.current = ammo.max;
        tf.translation.x = if tf.translation.x < 0.0 { -200.0 } else { 200.0 };
        tf.translation.y = 0.0;
        vel.x = 0.0;
        vel.y = 0.0;
    }

    round.phase = RoundPhase::Combat;
    round.round_number += 1;
    round.loser = None;
    println!("Round {} starting", round.round_number);
}
```

> **Ordering note:** `round_reset_system` must run after `apply_card_system`. Use `.chain()` in `add_systems`:
```rust
.add_systems(Update, (
    systems::draft::draft_offer_system,
    systems::draft::draft_select_system,
    systems::draft::apply_card_system,
    systems::draft::recalculate_stats_system,
    systems::lifecycle::round_reset_system,
).chain())
```

**Acceptance:** After picking a card, players reset to start positions, HP restored, projectiles cleared, new round begins.

---

## Step 16 — Match Over & Remaining Systems

**Goal:** When `ScoreBoard.wins[i] >= needed`, stop the loop.

**System (`systems/lifecycle.rs`):**
```rust
use bevy::prelude::*;
use crate::resources::{RoundPhase, RoundState, ScoreBoard};

pub fn match_over_system(
    score: Res<ScoreBoard>,
    mut round: ResMut<RoundState>,
) {
    if round.phase == RoundPhase::MatchOver { return; }
    if score.wins[0] >= score.needed || score.wins[1] >= score.needed {
        round.phase = RoundPhase::MatchOver;
        let winner = if score.wins[0] >= score.needed { 0 } else { 1 };
        println!("MATCH OVER — Player {} wins!", winner);
    }
}
```

**Final `Update` schedule order:**
```rust
.add_systems(Update, (
    // Combat phase
    systems::input::input_system,
    systems::input::jump_system,
    systems::input::facing_system,
    systems::input::block_system,
    systems::movement::gravity_system,
    systems::combat::shoot_system,
    systems::combat::projectile_move_system,
    systems::combat::projectile_lifetime_system,
    systems::combat::collision_system,
    systems::lifecycle::death_system,
).chain())
.add_systems(Update, (
    // Draft phase
    systems::draft::draft_offer_system,
    systems::draft::draft_select_system,
    systems::draft::apply_card_system,
    systems::draft::recalculate_stats_system,
    systems::lifecycle::round_reset_system,
    systems::lifecycle::match_over_system,
).chain())
```

**Acceptance:** After 5 wins by one player, console prints match winner and phase becomes `MatchOver`.

---

## Bevy 0.19 Migration Checklist for This Prototype

| Old API (pre-0.19) | 0.19 replacement | Where used |
|---|---|---|
| `Camera2dBundle::default()` | `Camera2d` | `setup`  |
| `SpriteBundle { sprite: Sprite { .. }, .. }` | `Sprite::from_color(color, size)` | `setup`, `shoot_system`  |
| `time.delta_seconds()` | `time.delta_secs()` | `gravity_system`, `projectile_move_system`  |
| `#[derive(Component, Resource)]` | Derive **only** `Resource` | All resources |
| `rand::thread_rng()` | `rand::rng()` | `draft_offer_system`  |
| `rand::seq::SliceRandom` | `rand::seq::IndexedRandom` | `draft_offer_system` |

---

## What to Verify Before Continuing

1. **`rand` version in Cargo.lock.** If you see `rand 0.8`, either pin `rand = "0.8"` and use `thread_rng()`, or update to `0.10` and use `rng()`. The 0.19 ecosystem examples use 0.10 .
2. **`Camera2d` compiles.** If not, check your `bevy` feature flags — `bevy_camera` may need explicit enabling depending on your `DefaultPlugins` configuration.
3. **Resource-as-component conflicts.** If any system panics with "resource already exists as component", check you haven't accidentally added `#[derive(Component)]` to a `Resource` type .

Once Steps 1–16 run without panics and the draft loop cycles correctly, you have a functional white-room ROUNDS prototype. The remaining work is **tuning** (card balance, physics feel, AI opponent) — not architecture.
