#pragma once

#include "common.h"
#include "engine.h"
#include "level.h"

struct BlockMovementThroughPeopleSystem
    : public OneOffSystem
    , public AccessEvents_Listen<CommandSignal>
    , public AccessWorld_UseUnique<Level>
    , public AccessWorld_UseUnique<CommandContext>
    , public AccessWorld_QueryAllEntitiesWith<Person, Name, WorldPosition>
    , public AccessWorld_QueryAllEntitiesWith<Player, WorldPosition>
    , public AccessWorld_QueryAllEntitiesWith<Blocked, WorldPosition>
    , public AccessWorld_QueryComponent<Player>
    , public AccessWorld_QueryComponent<BumpDefault>
    , public AccessWorld_QueryComponent<WorldPosition>
    , public AccessEvents_Emit<IssueCommandSignal>
{
    void react_to_event(CommandSignal& signal)
    {
        if (signal.type != CommandType::Move) return;
        
        auto& level = AccessWorld_UseUnique<Level>::access_unique();
        auto& context = AccessWorld_UseUnique<CommandContext>::access_unique();

        if (!AccessWorld_QueryComponent<Player>::has_component(context.subject))
        {
            for (auto&& [e, wp] : AccessWorld_QueryAllEntitiesWith<Player, WorldPosition>::query().each())
            {
                if (context.subject != e)
                {
                    if (signal.data.move.to_x == wp.x && signal.data.move.to_y == wp.y)
                    {
                        context.cancelled = true;
                        break;
                    }
                }
            }
        }

        if (context.cancelled) return;

        for (auto&& [e, p, n, wp] : AccessWorld_QueryAllEntitiesWith<Person, Name, WorldPosition>::query().each())
        {
            if (context.subject == e)
                continue;

            if (signal.data.move.to_x == wp.x && signal.data.move.to_y == wp.y)
            {                   
                wp.x = signal.data.move.from_x;
                wp.y = signal.data.move.from_y;

                break;
            }
        }

        for (auto&& [ e, wp ] : AccessWorld_QueryAllEntitiesWith<Blocked, WorldPosition>::query().each())
        {
            if (signal.data.move.to_x == wp.x && signal.data.move.to_y == wp.y)
            {
                if (AccessWorld_QueryComponent<BumpDefault>::has_component(e))
                {
                    auto& bump = AccessWorld_QueryComponent<BumpDefault>::get_component(e);

                    IssueCommandSignal issue;
                    issue.subject = context.subject;
                    issue.targets.push_back(e);
                    issue.type = bump.type;
                    issue.data = bump.data;
                    AccessEvents_Emit<IssueCommandSignal>::emit_event(issue);
                }

                context.cancelled = true;
                break;
            }
        }
    }
};

struct SymbolRenderSystem
    : public RuntimeSystem
    , public AccessWorld_UseUnique<Level>
    , public AccessWorld_QueryAllEntitiesWith<Symbol, WorldPosition>
    , public AccessWorld_QueryAllEntitiesWith<Person, Symbol, WorldPosition>
    , public AccessWorld_QueryAllEntitiesWith<Player, Symbol, WorldPosition>
    , public AccessWorld_QueryComponent<Health>
    , public AccessWorld_QueryComponent<Person>
    , public AccessWorld_QueryComponent<Player>
    , public AccessWorld_QueryComponent<Colored>
    , public AccessConsole
{
    void activate() override
    {
        auto& level = AccessWorld_UseUnique<Level>::access_unique();
        
        for (auto&& [entity, symbol, world_pos] : AccessWorld_QueryAllEntitiesWith<Symbol, WorldPosition>::query().each())
        {
            if (AccessWorld_QueryComponent<Person>::has_component(entity)) continue;
            if (AccessWorld_QueryComponent<Player>::has_component(entity)) continue;

            const ScreenPosition wp = { world_pos.x, world_pos.y };

            std::string s(1, level.regions[wp.x][wp.y]);
            
            if (level.map->isInFov(wp.x, wp.y))
            {
                if (AccessWorld_QueryComponent<Colored>::has_component(entity))
                {
                    fg(wp, AccessWorld_QueryComponent<Colored>::get_component(entity).color);
                }
                else
                {
                    fg(wp, "#ffffff"_rgb);
                }
                ch(wp, symbol.sym);
                level.memory[wp.x][wp.y] = symbol.sym[0];
            }
            else
            {
                fg(wp, HSL(level.hues[wp.x][wp.y], level.sats[wp.x][wp.y], 0.15f));
                std::string s(1, level.memory[wp.x][wp.y]);
                ch(wp, s);
            }
        }

        for (auto&& [entity, _, symbol, world_pos] : AccessWorld_QueryAllEntitiesWith<Person, Symbol, WorldPosition>::query().each())
        {
            const ScreenPosition wp = { world_pos.x, world_pos.y };

            std::string s(1, level.regions[wp.x][wp.y]);

            if (level.map->isInFov(wp.x, wp.y))
            {
                if (AccessWorld_QueryComponent<Colored>::has_component(entity))
                {
                    fg(wp, AccessWorld_QueryComponent<Colored>::get_component(entity).color);
                }
                else
                {
                    fg(wp, "#ffffff"_rgb);
                }
                ch(wp, symbol.sym);
                level.memory[wp.x][wp.y] = symbol.sym[0];
            }
        }

        for (auto&& [entity, symbol, world_pos] : AccessWorld_QueryAllEntitiesWith<Player, Symbol, WorldPosition>::query().each())
        {
            const ScreenPosition wp = { world_pos.x, world_pos.y };

            std::string s(1, level.regions[wp.x][wp.y]);
            if (AccessWorld_QueryComponent<Colored>::has_component(entity))
            {
                fg(wp, AccessWorld_QueryComponent<Colored>::get_component(entity).color);
            }
            else
            {
                fg(wp, "#ffffff"_rgb);
            }
            ch(wp, symbol.sym);
            level.memory[wp.x][wp.y] = symbol.sym[0];
        }
    }
};
