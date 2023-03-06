#include "engine.h"
#include "utils.h"
#include "graphs.h"
#include "level.h"

#include <unordered_map>

#undef main

struct LevelCreationEvent {};

struct LevelCreationSystem
    : public OneOffSystem
    , public AccessWorld_Unique<Level>
    , public AccessEvents_Listen<KeyEvent>
    , public AccessEvents_Emit<LevelCreationEvent>
{
    void activate() override
    {
        AccessWorld_Unique<Level>::access_unique().generate();
        AccessEvents_Emit<LevelCreationEvent>::emit_event();
    }

    void react_to_event(KeyEvent& signal)
    {
        if (signal.key == KeyCode::KEY_SPACE)
        {
            AccessWorld_Unique<Level>::access_unique().generate();
            AccessEvents_Emit<LevelCreationEvent>::emit_event();
        }
    }
};

struct Person {
    std::string name;
};

struct Place {};

struct Player {};

struct Symbol {
    std::string sym;
};

struct Health {
    int max_hp;
    int current_hp;
};

struct WorldPosition {
    int x;
    int y;
};

struct PlayerCreationSystem
    : public OneOffSystem
    , public AccessWorld_Unique<Level>
    , public AccessWorld_ModifyWorld
    , public AccessWorld_ModifyEntity
    , public AccessEvents_Listen<LevelCreationEvent>
{
    Entity last_player_entity = entt::null;

    void react_to_event(LevelCreationEvent& signal) override    
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

        AccessWorld_ModifyEntity::add_tag_component<Player>(last_player_entity);
        AccessWorld_ModifyEntity::add_component<Health>(last_player_entity, 5, 5);
        AccessWorld_ModifyEntity::add_component<Symbol>(last_player_entity, "@");
        AccessWorld_ModifyEntity::add_component<WorldPosition>(last_player_entity, pos.x, pos.y);
    }
};

struct Debug_PlayerDamageDealingSystem
    : public OneOffSystem
    , public AccessWorld_QueryByEntity<Player, Health>
    , public AccessEvents_Listen<KeyEvent>
{
    void react_to_event(KeyEvent& signal) override
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
};

struct SymbolRenderSystem
    : public RuntimeSystem
    , public AccessWorld_QueryByEntity<Symbol, WorldPosition>    
    , public AccessWorld_QueryByComponent<Health>
    , public AccessConsole
{
    void activate() override
    {
        for (auto&& [ entity, symbol, world_pos ] : AccessWorld_QueryByEntity<Symbol, WorldPosition>::query().each())
        {
            if (AccessWorld_QueryByComponent<Health>::has_component(entity))
            {
                const auto& hp = AccessWorld_QueryByComponent<Health>::get_component(entity);
                const float ratio = std::pow(1.0f - (float)(hp.current_hp - 1) / (float)hp.max_hp, 2.0f);
                fg({ world_pos.x, world_pos.y }, HSL(15.0f, ratio, 1.0f - ratio * 0.33f));
            }
            else
            {
                fg({ world_pos.x, world_pos.y }, "#f0f0f0"_rgb);
            }

            ch({ world_pos.x, world_pos.y }, symbol.sym);
        }
    }
};

struct MouseCursorSystem
    : public RuntimeSystem
    , public AccessMousePosition
    , public AccessConsole
{
    void activate() override
    {
        auto& mp = AccessMousePosition::get_mouse_position();
        bg(mp, HSL(200.0f, 1.0f, 0.5f));
    }
};

enum Debug_RenderMode
{
    Off = 0,
    RoomNumbers,
    Regions,
    COUNT
};

struct Debug_RoomLevelRenderSystem
    : public RuntimeSystem
    , public AccessConsole
    , public AccessEvents_Listen<KeyEvent>
    , public AccessWorld_Unique<Level>
{
    Debug_RenderMode mode = Debug_RenderMode::Off;

    void activate() override
    {
        if (mode == Debug_RenderMode::Off) return;

        const auto& level = access_unique();

        if (mode == Debug_RenderMode::RoomNumbers)
        {
            for (int i = 0; i < 80; i++)
            {
                for (int j = 0; j < 52; j++)
                {
                    if (level.rooms[i][j] != ' ')
                    {
                        std::string s(1, level.rooms[i][j]);
                        ch({ i, j }, s);
                    }
                }
            }
        }
        else if (mode == Debug_RenderMode::Regions)
        {
            for (int i = 0; i < 80; i++)
            {
                for (int j = 0; j < 52; j++)
                {
                    if (level.regions[i][j] != ' ')
                    {
                        std::string s(1, level.regions[i][j]);
                        ch({ i, j }, s);
                    }
                }
            }
        }
    }

    void react_to_event(KeyEvent& signal) override
    {
        if (signal.key == KeyCode::KEY_TAB)
        {
            mode = (Debug_RenderMode)(((int)mode + 1) % Debug_RenderMode::COUNT);
        }
    }
};

struct LevelRenderSystem
    : public RuntimeSystem    
    , public AccessConsole
    , public AccessWorld_Unique<Level>
{
    void activate() override
    {
        TCODRandom* rng = TCODRandom::getInstance();
        const auto& level = access_unique();
        
        for (int i = 0; i < 80; i++)
        {
            for (int j = 0; j < 52; j++)
            {
                float f = level.floor[i][j] * 0.5f;
                if (f > 0.1f && level.dig[i][j] != ' ')
                {
                    bg({ i, j }, HSL(rng->getFloat(160.0f, 190.0f), f, rng->getFloat(0.5f, 0.85f)));
                }
                else
                {
                    bg({ i, j }, RGB(0, 0, 0));
                }

                fg({ i, j }, RGB(128, 128, 128));
                
                std::string s(1, level.dig[i][j]);
                ch({ i, j }, s);
            }
        }
    }
};

int main(int argc, char* argv[])
{
    PoirogueEngine engine;
    engine.add_one_off_system<LevelCreationSystem>();
    engine.add_one_off_system<PlayerCreationSystem>(); 
    //engine.add_one_off_system<Debug_PlayerDamageDealingSystem>();

    engine.add_runtime_system<LevelRenderSystem>();
    engine.add_runtime_system<Debug_RoomLevelRenderSystem>();
    engine.add_runtime_system<SymbolRenderSystem>();
    engine.add_runtime_system<MouseCursorSystem>();

    engine.restart_game();
    
    while (engine) {
        engine.start_frame();
        engine.poll_events();
        engine.run_systems();
        engine.end_frame();
    }
}
