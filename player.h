#pragma once

#include "graphs.h"
#include "config.h"

struct Level;
struct LevelCreationEvent;
struct Player;
struct Health;

struct PlayerCreationSystem
    : public OneOffSystem
    , public AccessWorld_Unique<Level>
    , public AccessWorld_ModifyWorld
    , public AccessWorld_ModifyEntity
    , public AccessEvents_Listen<LevelCreationEvent>
{
    Entity last_player_entity = entt::null;

    void react_to_event(LevelCreationEvent& signal) override;
};

struct Debug_PlayerDamageDealingSystem
    : public OneOffSystem
    , public AccessWorld_QueryByEntity<Player, Health>
    , public AccessEvents_Listen<KeyEvent>
{
    void react_to_event(KeyEvent& signal) override;
};