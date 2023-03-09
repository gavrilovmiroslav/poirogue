#pragma once

#include "graphs.h"
#include "commands.h"

struct Level;
struct LevelCreationEvent;
struct Player;
struct Health;

struct PlayerCreationSystem
    : public OneOffSystem
    , public AccessWorld_UseUnique<Level>
    , public AccessWorld_ModifyWorld
    , public AccessWorld_ModifyEntity
    , public AccessEvents_Listen<LevelCreationEvent>
{
    Entity last_player_entity = entt::null;

    void react_to_event(LevelCreationEvent& signal) override;
};

struct PlayerChoiceSystem
    : public RuntimeSystem
    , public AccessWorld_QueryAllEntitiesWith<Player>
    , public AccessWorld_QueryComponent<WorldPosition>
    , public AccessWorld_QueryComponent<Sight>
    , public AccessWorld_UseUnique<Level>
    , public AccessEvents_Listen<AwaitingActionSignal>
    , public AccessEvents_Listen<KeyEvent>
    , public AccessEvents_Emit<IssueCommandSignal>
{
    bool player_turn = false;

    void react_to_event(AwaitingActionSignal& signal) override;
    void react_to_event(KeyEvent& key) override;

    void issue_command(IssueCommandSignal);
};