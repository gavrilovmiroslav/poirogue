#pragma once

#include "engine.h"

#include <any>
#include <unordered_map>

struct Level;
struct PeopleMapping;

struct WorldCrafting
    : public CraftingPipeline
    , public AccessWorld_UseUnique<Level>
    , public AccessWorld_UseUnique<PeopleMapping>
    , public AccessWorld_ModifyWorld
    , public AccessWorld_ModifyEntity
    , public AccessYAML    
{
    void execute_crafting() override;
        
    void create_warehouse(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_machine_shop(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_library(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_cleric(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_inn(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_temple(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_mess_hall(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_gymnasium(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_mineshaft(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_junkyard(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_foundry(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_skyport(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_lounge(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_market(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_drinking_hole(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_shrine(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_abandoned_warehouse(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_black_market(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_monolith(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_hidden_nook(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);
    void create_haunted_spot(Level& level, PeopleMapping& mapping, int region, std::vector<WorldPosition> tiles, WorldPosition center);

    Entity create_wares(WorldPosition tile, char sym = WIRE_SYM);
    Entity create_furnace(WorldPosition wp);
    Entity create_machine(WorldPosition wp);
    Entity create_bookshelf(WorldPosition wp);
    Entity create_chest(WorldPosition wp);
    
    void block_sight(Entity e, WorldPosition wp);
    void block_walking(Entity e, WorldPosition wp);
    void block_sight_walking(Entity e, WorldPosition wp);
};
