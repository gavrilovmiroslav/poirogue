#pragma once

#include "engine.h"
#include "common.h"

struct Level;
struct PeopleMapping;
struct Person;

struct PlotCrafting
    : public CraftingPipeline
    , public AccessWorld_UseUnique<Level>
    , public AccessWorld_UseUnique<PeopleMapping>
    , public AccessWorld_ModifyWorld
    , public AccessWorld_ModifyEntity
    , public AccessYAML
{
    void execute_crafting() override;
    
    void add_scenario(SocialInteraction s)
    {
        social_interactions.push_back(s);
    }
private:
    std::vector<SocialInteraction> social_interactions;
};