#pragma once

#include "common.h"
#include "engine.h"

struct Level;

struct PopulationCrafting
    : public CraftingPipeline
    , public AccessWorld_UseUnique<Level>
    , public AccessWorld_UseUnique<PeopleMapping>
    , public AccessWorld_ModifyWorld
    , public AccessWorld_ModifyEntity
    , public AccessWorld_QueryAllEntitiesWith<Person>
    , public AccessYAML
{
    void generate_people_graph();
    void execute_crafting() override;
};