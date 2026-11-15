//! Типы, общие для клиента и сервера.

use bevy::prelude::*;
use bevy::reflect::Reflect;
use leafwing_input_manager::Actionlike;
use lightyear::connection::id::ClientId; // <-- Правильный путь
use serde::{Deserialize, Serialize};

#[derive(
    Component, Serialize, Deserialize, Debug, Clone, Copy, Default,
    PartialEq, Eq, Hash, Reflect, Actionlike,
)]
#[reflect(Component)]
pub enum PlayerInput {
    #[default]
    None,
    Jump,
    Shoot,
    Block,
}

#[derive(Component)]
pub struct NetworkedPlayer {
    pub client_id: ClientId,
}

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct NetworkedHealth {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct NetworkedVelocity {
    pub x: f32,
    pub y: f32,
}

pub const SERVER_PORT: u16 = 5000;z