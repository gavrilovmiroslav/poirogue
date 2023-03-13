#include "people.h"
#include "common.h"
#include "graphs.h"
#include "level.h"

#include <yaml-cpp/yaml.h>

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

void populate_queues(YAML::Node places_yaml, std::unordered_map<std::string, PlaceWeightQueue>& queues)
{
    TCODRandom* rng = TCODRandom::getInstance();

    for (auto key : {
    "few-work-no-visits",
    "few-work-many-visits",
    "many-work-few-visits",
    "some-work-some-visit",
    "no-work-only-visit",
    "exceptional-visit"
        }) {
        PlaceWeightQueue queue{};

        for (auto pplace : places_yaml[key])
        {
            for (auto place : pplace) {
                auto key = place.first;
                auto value = place.second;
                queue.push({ key.as<std::string>(), value.as<int>() + rng->getInt(-5, 0) });
            }
        }
        queues.erase(key);
        queues.insert({ std::string(key), queue });
    }
}

void PopulationCrafting::generate_people_graph()
{
    auto& peopleMapping = AccessWorld_UseUnique<PeopleMapping>::access_unique();
    peopleMapping.graph = std::shared_ptr<graphs::Graph>(new graphs::Graph());
    auto& places = peopleMapping.places;
    auto& people = peopleMapping.people;
    auto& graph = peopleMapping.graph;

    TCODRandom* rng = TCODRandom::getInstance();
    std::unordered_set<int> used_places;

    for (int i = 0; i < REGION_COUNT; i++)
    {
        places[i] = graph->create_node();
        graph->label_node(places[i], "place #" + std::to_string(i + 1));
        graph->tag_node<Place>(places[i], i);
    }

    for (int i = 0; i < PEOPLE_COUNT + 1; i++)
    {
        people[i] = graph->create_node();
        graph->tag_node<Person>(people[i], i);
        graph->label_node(people[i], "person #" + std::to_string(i + 1));

        int p = rng->getInt(0, REGION_COUNT - 1);
        auto lives_in_arrow = graph->create_arrow(people[i], places[p]);
        graph->label_edge(lives_in_arrow, "lives near");
        peopleMapping.residents[p].living.push_back(i);
        graph->tag_edge<LivesIn>(lives_in_arrow);
        used_places.insert(p);

        p = rng->getInt(0, REGION_COUNT - 1);
        auto works_in_arrow = graph->create_arrow(people[i], places[p]);
        graph->label_edge(works_in_arrow, "works in");
        peopleMapping.residents[p].working.push_back(i);
        graph->tag_edge<WorksIn>(lives_in_arrow);
        used_places.insert(p);
    }

    int current_person = 0;
    for (int j = 0; j < REGION_COUNT; j++)
    {
        if (used_places.count(j) > 0 || rng->getInt(0, 100) > 85) continue;
        int ps = std::max(PEOPLE_COUNT - 1, 2 + rng->getInt(0, PEOPLE_COUNT - 1));

        for (int i = 0; i < ps; i++)
        {
            auto uses_arrow = graph->create_arrow(people[current_person % PEOPLE_COUNT], places[j]);
            graph->label_edge(uses_arrow, "visits");
            graph->tag_edge<Visits>(uses_arrow);
            peopleMapping.residents[j].visits.push_back(current_person % PEOPLE_COUNT);
            current_person++;
        }
    }

    current_person = 0;
    for (int j = 0; j < REGION_COUNT; j++)
    {
        if (rng->getInt(0, 100) > 95) continue;
        int ps = std::max(PEOPLE_COUNT, 2 + rng->getInt(0, PEOPLE_COUNT));

        for (int i = 0; i < ps; i++)
        {
            if (rng->getInt(0, 100) > 30) continue;
            auto uses_arrow = graph->create_arrow(people[current_person % (PEOPLE_COUNT + 1)], places[j]);
            graph->label_edge(uses_arrow, "visits");
            graph->tag_edge<Visits>(uses_arrow);
            peopleMapping.residents[j].visits.push_back((current_person * rng->getInt(1, 10)) % (PEOPLE_COUNT + 1));
            current_person++;
        }
    }

    graph->label_node(peopleMapping.people[PEOPLE_COUNT], "victim");

    std::unordered_map<std::string, PlaceWeightQueue> queues;
    auto places_yaml = AccessYAML::load("data/lists/places.yaml");
    auto jobs_yaml = AccessYAML::load("data/lists/jobs.yaml");

    populate_queues(places_yaml, queues);

    for (int i = 0; i < REGION_COUNT; i++)
    {
        const int w = peopleMapping.residents[i].working.size();
        const int l = peopleMapping.residents[i].living.size();
        const int v = peopleMapping.residents[i].visits.size();

        auto kind = PlaceKind::SomeVisits;
        if (rng->getInt(0, 100) > 80)
        {            
            if (w == 0 && v > 0)
            {
                kind = PlaceKind::NoWorkOnlyVisit;
            }
            else if (w > 0 && v == 0)
            {
                kind = PlaceKind::FewWorkNoVisits;
            }
            else if (w > 0 && v > 0 && v > rng->getFloat(1.0f, 2.0f) * w)
            {
                kind = PlaceKind::FewWorkManyVisits;
            }
            else if (w > 0 && v > 0 && w > rng->getFloat(1.0f, 2.0f) * v)
            {
                kind = PlaceKind::ManyWorkFewVisits;
            }
            else if (w > 0 && v > 0)
            {
                kind = PlaceKind::SomeWorkSomeVisit;
            }
        }
        else
        {
            kind = (PlaceKind)rng->getInt(0, (int)PlaceKind::COUNT - 1);
        }

        auto place_kind = std::string(get_place_kind(kind));
        if (queues[place_kind].empty())
        {
            populate_queues(places_yaml, queues);
            assert(!queues[place_kind].empty());
        }

        auto place = queues[place_kind].top();
        queues[place_kind].pop();
        auto value = std::get<0>(place);
        auto prio = std::get<1>(place);

        auto job_roles = jobs_yaml[value];

        std::transform(value.begin(), value.end(), value.begin(), ::toupper);
        graph->label_node(places[i], value);

        for (auto res : peopleMapping.residents[i].working)
        {
            if (job_roles.size() > 0)
            {
                auto job_role_index = rng->getInt(0, job_roles.size() - 1);
                graph->tag_node<Job>(people[res], job_roles[job_role_index].as<std::string>());
            }
            else
            {
                graph->tag_node<Job>(people[res], "VISITOR");
            }
        }
    }
}

void PopulationCrafting::execute_crafting()
{
    auto& peopleMapping = AccessWorld_UseUnique<PeopleMapping>::access_unique();

    std::unordered_set<int> used_spaces;

    TCODRandom* rng = TCODRandom::getInstance();

    AccessWorld_UseUnique<Level>::access_unique().generate();
    auto& people_mapping = AccessWorld_UseUnique<PeopleMapping>::access_unique();
    people_mapping.graph.reset();

    auto queried = AccessWorld_QueryAllEntitiesWith<Person>::query();
    AccessWorld_ModifyWorld::destroy_entities(queried.begin(), queried.end());

    generate_people_graph();

    auto names = AccessYAML::load("data/lists/names.yaml");

    auto female_name_list = names["female-names"];
    auto male_name_list = names["male-names"];
    std::vector<char> letters{ 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'r', 's', 't', 'u', 'w', 'v', 'y' };

    auto& level = AccessWorld_UseUnique<Level>::access_unique();
    for (auto person : people_mapping.people)
    {
        const int person_id = people_mapping.graph->get_tag<Person>(person).person_id;
        for (auto edge : people_mapping.graph->get_all_edges(person))
        {
            if (people_mapping.graph->has_tag<LivesIn>(edge))
            {
                auto place_node = people_mapping.graph->get_target(edge);
                const int region = people_mapping.graph->get_tag<Place>(place_node).place_id;

                auto& tile = level.region_tiles[region][rng->getInt(0, level.region_tiles[region].size() - 1)];
                while (used_spaces.count(TO_XY(tile.x, tile.y) > 0)) {
                    tile = level.region_tiles[region][rng->getInt(0, level.region_tiles[region].size() - 1)];
                }

                used_spaces.insert(TO_XY(tile.x, tile.y));

                Sex sex = (rng->getInt(0, 101) >= 50 ? Sex::Female : Sex::Male);

                // create person

                auto game_person = AccessWorld_ModifyWorld::create_entity();
                AccessWorld_ModifyEntity::add_component<Person>(game_person, person_id);
                AccessWorld_ModifyEntity::add_component<Sex>(game_person, sex);
                AccessWorld_ModifyEntity::add_component<Health>(game_person, 100, 100);
                AccessWorld_ModifyEntity::add_component<ActionPoints>(game_person, 0);
                AccessWorld_ModifyEntity::add_component<Speed>(game_person, rng->getInt(80, 110));

                auto job = people_mapping.graph->get_tag<Job>(person).role;
                AccessWorld_ModifyEntity::add_component<Job>(game_person, job);
                int letter_index = rng->getInt(0, letters.size() - 1);
                char c = letters[letter_index];
                std::string s_low(1, c);
                letters.erase(letters.begin() + letter_index);

                YAML::Node name_list = (sex == Sex::Female ? female_name_list : male_name_list)[s_low];
                int name_index = rng->getInt(0, name_list.size() - 1);
                auto name = name_list[name_index].as<std::string>();
                name_list.remove(name_index);
                people_mapping.graph->label_node(person, name + "(" + job + ")");

                std::string s_high(1, c - 32);
                AccessWorld_ModifyEntity::add_component<Symbol>(game_person, s_high);
                AccessWorld_ModifyEntity::add_component<Name>(game_person, name);
                AccessWorld_ModifyEntity::add_component<AIPlayer>(game_person);
                AccessWorld_ModifyEntity::add_component<WorldPosition>(game_person, (int)tile.x, (int)tile.y);
            }
        }
    }

    people_mapping.graph->print();
}