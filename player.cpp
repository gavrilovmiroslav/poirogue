#include "player.h"

#include "config.h"
#include "common.h"
#include "level.h"
#include "engine.h"

void PlayerCreationSystem::react_to_event(LevelCreationEvent& signal)
{
    if (last_player_entity != entt::null)
    {
        AccessWorld_ModifyWorld::destroy_entity(last_player_entity);
    }

    TCODRandom* rng = TCODRandom::getInstance();
    last_player_entity = AccessWorld_ModifyWorld::create_entity();

    const auto& level = AccessWorld_UseUnique<Level>::access_unique();
    const auto size = level.walkable.size();
    const auto pos = level.walkable[rng->getInt(0, size)];

    this->AccessWorld_ModifyEntity::add_tag_component<Player>(last_player_entity);
    this->AccessWorld_ModifyEntity::add_component<Health>(last_player_entity, 100, 100);
    this->AccessWorld_ModifyEntity::add_component<Name>(last_player_entity, "DETECTIVE");
    this->AccessWorld_ModifyEntity::add_component<ActionPoints>(last_player_entity, 0);
    this->AccessWorld_ModifyEntity::add_component<Speed>(last_player_entity, ATTRIBUTE_SPEED_NORM);
    this->AccessWorld_ModifyEntity::add_component<Symbol>(last_player_entity, "@");
    this->AccessWorld_ModifyEntity::add_component<WorldPosition>(last_player_entity, pos.x, pos.y);
    auto& sight = this->AccessWorld_ModifyEntity::add_component<Sight>(last_player_entity, ATTRIBUTE_SIGHT_NORM);

    level.map->computeFov(pos.x, pos.y, sight.radius, true, FOV_RESTRICTIVE);
}

void PlayerChoiceSystem::react_to_event(AwaitingActionSignal& signal)
{
    const auto player = AccessWorld_QueryAllEntitiesWith<Player>::query().front();
    if (signal.current_in_order == player)
    {
        player_turn = true;
    }
}

void PlayerChoiceSystem::issue_command(IssueCommandSignal issue)
{
    player_turn = false;
    AccessEvents_Emit<IssueCommandSignal>::emit_event(issue);
}

void PlayerChoiceSystem::react_to_event(KeyEvent& key)
{
    if (!player_turn)
        return;

    const auto player = AccessWorld_QueryAllEntitiesWith<Player>::query().front();
    const auto& level = AccessWorld_UseUnique<Level>::access_unique();
    const auto& sight = AccessWorld_QueryComponent<Sight>::get_component(player);
    auto& world_pos = AccessWorld_QueryComponent<WorldPosition>::get_component(player);

    int dx = 0;
    int dy = 0;

    IssueCommandSignal issue;

    switch (key.key)
    {
    case KeyCode::KEY_PERIOD:
        issue.subject = player;
        issue.type = CommandType::Wait;
        issue_command(issue);

        return;

    case KeyCode::KEY_A:
        dx = -1;
        break;

    case KeyCode::KEY_D:
        dx = 1;
        break;

    case KeyCode::KEY_W:
        dy = -1;
        break;

    case KeyCode::KEY_S:
        dy = 1;
        break;

    case KeyCode::KEY_Q:
        dx = -1;
        dy = -1;
        break;

    case KeyCode::KEY_E:
        dx = 1;
        dy = -1;
        break;

    case KeyCode::KEY_Z:
        dx = -1;
        dy = 1;
        break;

    case KeyCode::KEY_X: case KeyCode::KEY_C:
        dx = 1;
        dy = 1;
        break;
    }

    if (dx != 0 || dy != 0)
    {
        const int x = world_pos.x + dx;
        const int y = world_pos.y + dy;

        issue.subject = player;
        issue.type = CommandType::Move;
        issue.data.move.from_x = world_pos.x;
        issue.data.move.from_y = world_pos.y;
        issue.data.move.to_x = x;
        issue.data.move.to_y = y;

        issue_command(issue);
    }
}