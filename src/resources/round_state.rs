use bevy::prelude::*;

/// Placeholder resources. Real fields arrive in Steps 7-10; they exist now
/// because `main.rs` (Step 1) registers all five resources at startup.
/// NOTE (Bevy 0.19): derive ONLY `Resource` — it is a subtrait of `Component`.
#[derive(Resource, Default)]
pub struct RoundState;

#[derive(Resource, Default)]
pub struct ScoreBoard;

#[derive(Resource, Default)]
pub struct DraftState;

#[derive(Resource, Default)]
pub struct CardPool;
