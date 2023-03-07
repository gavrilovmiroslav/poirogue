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

    const auto& level = AccessWorld_Unique<Level>::access_unique();
    const auto size = level.walkable.size();
    const auto pos = level.walkable[rng->getInt(0, size)];

    this->AccessWorld_ModifyEntity::add_tag_component<Player>(last_player_entity);
    this->AccessWorld_ModifyEntity::add_component<Health>(last_player_entity, 5, 5);
    this->AccessWorld_ModifyEntity::add_component<Symbol>(last_player_entity, "@");
    this->AccessWorld_ModifyEntity::add_component<WorldPosition>(last_player_entity, pos.x, pos.y);
}

void Debug_PlayerDamageDealingSystem::react_to_event(KeyEvent& signal)
{
    if (signal.key == KeyCode::KEY_D)
    {
        for (auto&& [entity, health] : AccessWorld_QueryByEntity<Player, Health>::query().each())
        {
            health.current_hp--;
            if (health.current_hp < 0) health.current_hp = 0;
        }
    }
}
