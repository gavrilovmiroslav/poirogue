#include "common.h"
#include "graphs.h"
#include "level.h"

std::vector<Entity> PeopleMapping::get_all_places_shuffled()
{
    std::vector<Entity> places;

    for (int i = 0; i < REGION_COUNT; i++)
    {
        places.push_back(this->places[i]);
    }

    shuffle(places);
    return places;
}

std::vector<Entity> PeopleMapping::get_all_people_shuffled()
{
    std::vector<Entity> people;
    
    for (int i = 0; i < PEOPLE_COUNT; i++)
    {
        people.push_back(this->people[i]);
    }

    shuffle(people);
    return people;
}

std::vector<Entity> PeopleMapping::get_all_visiting_with(Entity visitor, bool with_victim)
{
    std::unordered_set<Entity> all_visitors;
    for (auto edge : graph->get_all_edges(visitor))
    {
        if (graph->has_tag<Visits>(edge))
        {
            auto place = graph->get_target(edge);
            auto index = graph->get_tag<Place>(place).place_id;
            for (auto res : residents[index].visits)
            {
                if (people[res] != visitor && (with_victim || res != PEOPLE_COUNT))
                    all_visitors.insert(people[res]);
            }
        }
    }

    std::vector<Entity> vector;
    vector.insert(vector.end(), all_visitors.begin(), all_visitors.end());
    return vector;
}

std::vector<Entity> PeopleMapping::get_all_working_with(Entity visitor, bool with_victim)
{
    std::unordered_set<Entity> all_visitors;
    for (auto edge : graph->get_all_edges(visitor))
    {
        if (graph->has_tag<Visits>(edge))
        {
            auto place = graph->get_target(edge);
            auto index = graph->get_tag<Place>(place).place_id;
            for (auto res : residents[index].working)
            {
                if (people[res] != visitor && (with_victim || res != PEOPLE_COUNT))
                    all_visitors.insert(people[res]);
            }
        }
    }

    std::vector<Entity> vector;
    vector.insert(vector.end(), all_visitors.begin(), all_visitors.end());
    return vector;
}

std::vector<Entity> PeopleMapping::get_all_living_with(Entity visitor, bool with_victim)
{
    std::unordered_set<Entity> all_visitors;
    for (auto edge : graph->get_all_edges(visitor))
    {
        if (graph->has_tag<Visits>(edge))
        {
            auto place = graph->get_target(edge);
            auto index = graph->get_tag<Place>(place).place_id;
            for (auto res : residents[index].living)
            {
                if (people[res] != visitor && (with_victim || res != PEOPLE_COUNT))
                    all_visitors.insert(people[res]);
            }
        }
    }

    std::vector<Entity> vector;
    vector.insert(vector.end(), all_visitors.begin(), all_visitors.end());
    return vector;
}
