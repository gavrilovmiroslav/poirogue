#pragma once

#include "common.h"
#include "engine.h"
#include "level.h"

struct SymbolRenderSystem
    : public RuntimeSystem
    , public AccessWorld_UseUnique<Level>
    , public AccessWorld_QueryAllEntitiesWith<Symbol, WorldPosition>
    , public AccessWorld_QueryComponent<Health>
    , public AccessConsole
{
    void activate() override
    {
        auto& level = AccessWorld_UseUnique<Level>::access_unique();
        for (auto&& [entity, symbol, world_pos] : AccessWorld_QueryAllEntitiesWith<Symbol, WorldPosition>::query().each())
        {
            if (level.map->isInFov(world_pos.x, world_pos.y))
            {
                fg({ world_pos.x, world_pos.y }, "#ffffff"_rgb);
                ch({ world_pos.x, world_pos.y }, symbol.sym);
                level.memory[world_pos.x][world_pos.y] = symbol.sym[0];
            }
            else
            {
                fg({ world_pos.x, world_pos.y }, "#a0a0a0"_rgb);
                std::string s(1, level.memory[world_pos.x][world_pos.y]);
                ch({ world_pos.x, world_pos.y }, s);
            }
        }
    }
};
