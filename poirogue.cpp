
#include "config.h"
#include "common.h"
#include "engine.h"
#include "utils.h"
#include "graphs.h"
#include "level.h"
#include "player.h"
#include "people.h"

#include <unordered_map>

#undef main

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

int main(int argc, char* argv[])
{
    PoirogueEngine engine;
    engine.add_one_off_system<LevelCreationSystem>();
    engine.add_one_off_system<PlayerCreationSystem>(); 
    //engine.add_one_off_system<Debug_PlayerDamageDealingSystem>();

    engine.add_runtime_system<LevelRenderSystem>();
    engine.add_runtime_system<ShimmerRenderSystem>();
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
