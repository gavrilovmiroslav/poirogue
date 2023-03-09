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
        for (auto&& [entity, symbol, world_pos] : AccessWorld_QueryAllEntitiesWith<Symbol, WorldPosition>::query().each())
        {
            if (AccessWorld_UseUnique<Level>::access_unique().map->isInFov(world_pos.x, world_pos.y))
            {
                fg({ world_pos.x, world_pos.y }, "#ffffff"_rgb);
                ch({ world_pos.x, world_pos.y }, symbol.sym);
            }
        }
    }
};
