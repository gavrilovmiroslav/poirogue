#pragma once

#include "graphs.h"
#include "config.h"

struct Person {
    int person_id;
    std::string name;
};

struct Place { int place_id; };

struct WorksIn {};
struct LivesIn {};
struct Visits {};

struct PeopleMapping
{
    graphs::NodeEntity people[PEOPLE_COUNT];
    graphs::NodeEntity places[REGION_COUNT];
    graphs::Graph graph;
};

PeopleMapping generate_people_graph() {
    PeopleMapping map;
    auto& places = map.places;
    auto& people = map.people;
    auto& graph = map.graph;

    TCODRandom* rng = TCODRandom::getInstance();    
    std::unordered_set<int> used_places;

    for (int i = 0; i < REGION_COUNT; i++)
    {
        places[i] = graph.create_node();
        graph.label_node(places[i], "place #" + std::to_string(i + 1));
        graph.tag_node<Place>(places[i], i);
    }

    for (int i = 0; i < PEOPLE_COUNT; i++)
    {
        people[i] = graph.create_node();
        graph.tag_node<Person>(people[i], i);
        graph.label_node(people[i], "person #" + std::to_string(i + 1));

        int p = rng->getInt(0, REGION_COUNT - 1);
        auto lives_in_arrow = graph.create_arrow(people[i], places[p]);
        graph.label_edge(lives_in_arrow, "lives in");
        graph.tag_edge<LivesIn>(lives_in_arrow);
        used_places.insert(p);

        p = rng->getInt(0, REGION_COUNT - 1);
        auto works_in_arrow = graph.create_arrow(people[i], places[p]);
        graph.label_edge(works_in_arrow, "works in");
        graph.tag_edge<WorksIn>(lives_in_arrow);
        used_places.insert(p);
    }

    int current_person = 0;
    for (int j = 0; j < REGION_COUNT; j++)
    {
        if (used_places.count(j) > 0 || rng->getInt(0, 100) > 85) continue;
        int ps = std::max(PEOPLE_COUNT - 1, 2 + rng->getInt(0, PEOPLE_COUNT - 1));

        for (int i = 0; i < ps; i++)
        {
            auto uses_arrow = graph.create_arrow(people[current_person % PEOPLE_COUNT], places[j]);
            graph.label_edge(uses_arrow, "visits");
            graph.tag_edge<Visits>(uses_arrow);
            current_person++;
        }
    }

    graph.print();

    return map;
}