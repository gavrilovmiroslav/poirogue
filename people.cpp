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

std::vector<std::tuple<PersonEntity, PlaceEntity>> PeopleMapping::get_all_visiting_with(Entity visitor, bool with_victim)
{
    std::unordered_set<std::tuple<PersonEntity, PlaceEntity>, PersonPlaceHash> all_visitors;
    for (auto edge : graph->get_all_edges(visitor))
    {
        if (graph->has_tag<Visits>(edge))
        {
            auto place = graph->get_target(edge);
            auto index = graph->get_tag<Place>(place).place_id;
            for (auto res : residents[index].visits)
            {
                if (people[res] != visitor && (with_victim || res != PEOPLE_COUNT))
                    all_visitors.insert({ people[res], place });
            }
        }
    }

    std::vector<std::tuple<PersonEntity, PlaceEntity>> vector;
    vector.insert(vector.end(), all_visitors.begin(), all_visitors.end());
    return vector;
}

std::vector<std::tuple<PersonEntity, PlaceEntity>> PeopleMapping::get_all_working_with(Entity visitor, bool with_victim)
{
    std::unordered_set<std::tuple<PersonEntity, PlaceEntity>, PersonPlaceHash> all_visitors;
    for (auto edge : graph->get_all_edges(visitor))
    {
        if (graph->has_tag<Visits>(edge))
        {
            auto place = graph->get_target(edge);
            auto index = graph->get_tag<Place>(place).place_id;
            for (auto res : residents[index].working)
            {
                if (people[res] != visitor && (with_victim || res != PEOPLE_COUNT))
                    all_visitors.insert({ people[res], place });
            }
        }
    }

    std::vector<std::tuple<PersonEntity, PlaceEntity>> vector;
    vector.insert(vector.end(), all_visitors.begin(), all_visitors.end());
    return vector;
}

std::vector<std::tuple<PersonEntity, PlaceEntity>> PeopleMapping::get_all_living_with(Entity visitor, bool with_victim)
{
    std::unordered_set<std::tuple<PersonEntity, PlaceEntity>, PersonPlaceHash> all_visitors;
    for (auto edge : graph->get_all_edges(visitor))
    {
        if (graph->has_tag<Visits>(edge))
        {
            auto place = graph->get_target(edge);
            auto index = graph->get_tag<Place>(place).place_id;
            for (auto res : residents[index].living)
            {
                if (people[res] != visitor && (with_victim || res != PEOPLE_COUNT))
                    all_visitors.insert({ people[res], place });
            }
        }
    }

    std::vector<std::tuple<PersonEntity, PlaceEntity>> vector;
    vector.insert(vector.end(), all_visitors.begin(), all_visitors.end());
    return vector;
}

std::vector<std::tuple<PersonEntity, PlaceEntity>> PeopleMapping::get_all_related_with(Entity visitor, bool with_victim)
{
    std::unordered_set<std::tuple<PersonEntity, PlaceEntity>, PersonPlaceHash> all_visitors;
    for (auto edge : graph->get_all_edges(visitor))
    {
        if (graph->has_tag<Visits>(edge))
        {
            auto place = graph->get_target(edge);
            auto index = graph->get_tag<Place>(place).place_id;
            for (auto res : residents[index].living)
            {
                if (people[res] != visitor && (with_victim || res != PEOPLE_COUNT))
                    all_visitors.insert({ people[res], place });
            }

            for (auto res : residents[index].working)
            {
                if (people[res] != visitor && (with_victim || res != PEOPLE_COUNT))
                    all_visitors.insert({ people[res], place });
            }

            for (auto res : residents[index].visits)
            {
                if (people[res] != visitor && (with_victim || res != PEOPLE_COUNT))
                    all_visitors.insert({ people[res], place });
            }
        }
    }

    std::vector<std::tuple<PersonEntity, PlaceEntity>> vector;
    vector.insert(vector.end(), all_visitors.begin(), all_visitors.end());
    return vector;
}

Entity PeopleMapping::create_topic(std::string label, int event_id)
{
    auto topic_node = graph->create_node();
    graph->label_node(topic_node, label);
    graph->tag_node<CaseElement>(topic_node, CaseElement::TOPIC);
    graph->tag_node<CaseEvent>(topic_node, event_id);
    return topic_node;
}

Entity PeopleMapping::create_event(std::string label, int time, int event_id)
{
    auto event_node = graph->create_node();
    graph->label_node(event_node, label);    
    graph->tag_node<Time>(event_node, time);
    graph->tag_node<CaseElement>(event_node, CaseElement::EVENT); 
    graph->tag_node<CaseEvent>(event_node, event_id);
    return event_node;
}

void PeopleMapping::connect(Entity a, Entity b, std::string label, int event_id) {
    auto arrow = graph->create_arrow(a, b);
    graph->label_edge(arrow, label);
    graph->tag_edge<CaseEvent>(arrow, event_id);
}