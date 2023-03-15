#include "player.h"

#include "config.h"
#include "common.h"
#include "level.h"
#include "engine.h"

void PlayerCreationSystem::activate()
{
    AccessWorld_UseUnique<GameContext>::access_unique() = GameContext::Game;
}

void PlayerCreationSystem::react_to_event(LevelCreationEvent& signal)
{
    if (last_player_entity != entt::null)
    {
        destroy_entity(last_player_entity);
    }

    TCODRandom* rng = TCODRandom::getInstance();
    last_player_entity = create_entity();

    const auto& level = AccessWorld_UseUnique<Level>::access_unique();
    const auto size = level.walkable.size();
    const auto pos = level.walkable[rng->getInt(0, size)];

    add_tag_component<Player>(last_player_entity);
    add_component<Health>(last_player_entity, 100, 100);
    add_component<Name>(last_player_entity, "DETECTIVE");
    add_component<ActionPoints>(last_player_entity, 0);
    add_component<Speed>(last_player_entity, ATTRIBUTE_SPEED_NORM);
    add_component<Symbol>(last_player_entity, "@");
    add_component<WorldPosition>(last_player_entity, pos.x, pos.y);
    auto& inventory = add_component<Inventory>(last_player_entity);

    for (int i = 0; i < INVENTORY_SIZE; i++)
    {
        inventory.stuff[i] = entt::null;
    }

    {
        auto item_entity = create_entity();
        add_component<Item>(item_entity, "TRUTHSAYER MONOCLE");
        std::string s(1, MONO_SYM);
        add_component<Symbol>(item_entity, s);
        inventory.stuff[0] = item_entity;
    }
    
    {
        auto item_entity = create_entity();
        add_component<Item>(item_entity, "RING OF GLAMOR");
        std::string s(1, RING_SYM);
        add_component<Symbol>(item_entity, s);
        inventory.stuff[1] = item_entity;
    }

    {
        auto item_entity = create_entity();
        add_component<Item>(item_entity, "EYEDROPS OF BARTER");
        std::string s(1, EYEDROP_SYM);
        add_component<Symbol>(item_entity, s);
        inventory.stuff[2] = item_entity;
    }

    auto& sight = add_component<Sight>(last_player_entity, ATTRIBUTE_SIGHT_NORM);

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

    if (AccessWorld_UseUnique<GameContext>::access_unique() != GameContext::Game)
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