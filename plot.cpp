#include "plot.h"

#include "level.h"

void PlotCrafting::execute_crafting()
{
    auto& peopleMapping = AccessWorld_UseUnique<PeopleMapping>::access_unique();

    TCODRandom* rng = TCODRandom::getInstance();

    int event_id = 0;
    int size = (int)social_interactions.size();

    if (size == 0)
        return;

    int murder_index = rng->getInt(0, size - 1);
    social_interactions[murder_index](peopleMapping, event_id++, entt::null, true);

    for (int i = 0; i < social_interactions.size(); i++)
        if (i != murder_index)
            social_interactions[i](peopleMapping, event_id++, entt::null, false);

    auto people = peopleMapping.get_all_people_shuffled();
    for (int i = 0; i < social_interactions.size(); i++)
    {
        social_interactions[i](peopleMapping, event_id++, people.back(), false);
        people.pop_back();
    }
}
