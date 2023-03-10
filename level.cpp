#include "level.h"

#include "config.h"
#include "common.h"

#include <unordered_map>
#include <yaml-cpp/yaml.h>

void Level::init()
{
    walkable.clear();

    exploded_bombs.clear();

    for (int i = 0; i < WIDTH; i++)
    {
        for (int j = 0; j < HEIGHT; j++)
        {
            dig[i][j] = ' ';
            rooms[i][j] = ' ';
        }
    }
}

void Level::gradient()
{
    TCODRandom* rng = TCODRandom::getInstance();

    for (int i = 0; i < WIDTH; i++)
    {
        for (int j = 0; j < HEIGHT; j++)
        {
            dig[i][j] = ' ';
            digability[i][j] = rng->getFloat(0.0f, 1.0f, 0.5f);
        }
    }

    float f = 1.0f;
    float radius = 10.0f;
    for (int i = 0; i < 450; i++)
    {
        f *= 0.9991f;
        int x = rng->getInt(5, WIDTH - 5);
        int y = rng->getInt(5, HEIGHT - 5);

        for (int i = -(int)radius; i < (int)radius; i++)
        {
            for (int j = -(int)radius; j < (int)radius; j++)
            {
                if (x + i < 0 || x + i > WIDTH - 1) continue;
                if (y + j < 0 || y + j > HEIGHT - 1) continue;

                digability[x + i][y + j] *= f;
            }
        }

        radius *= 0.995f;
        if (radius < 1.0f)
        {
            break;
        }
    }
}

void Level::flood_fill(int bomb_count)
{
    TCODRandom* rng = TCODRandom::getInstance();

    bombs.reset();
    for (int i = 0; i < bomb_count; i++)
    {
        int8_t x = rng->getInt(5, WIDTH - 5);
        int8_t y = rng->getInt(5, HEIGHT - 5);
        bombs.set(TO_XY(x, y));
    }

    flood_fill_candidate.reset();
    flood_fill_visited.reset();

    while (!flood_fill_freelist.empty())
    {
        auto next = flood_fill_freelist.front();
        flood_fill_freelist.pop_front();

        dig[next.x][next.y] = '.';
        digability[next.x][next.y] = 0.0f;
        flood_fill_visited.set(TO_XY(next.x, next.y));

        for (int8_t i = -1; i < 2; i++)
        {
            if (next.x + i < 0) continue;
            if (next.x + i >= WIDTH - 1) continue;

            for (int8_t j = -1; j < 2; j++)
            {
                if (next.y + j < 0) continue;
                if (next.y + j >= HEIGHT - 1) continue;

                if (i == 0 && j == 0) continue;

                int8_t x = (int8_t)(next.x + i);
                int8_t y = (int8_t)(next.y + j);

                int ixy = TO_XY(x, y);

                if (flood_fill_candidate.test(ixy)) continue;

                if (!flood_fill_visited.test(ixy))
                {
                    if (bombs.test(ixy))
                    {
                        bombs.reset(ixy);
                        exploded_bombs.push_back(XY{ x, y });

                        int impact = rng->getInt(2, 5);
                        for (int8_t k = -impact; k < impact + 1; k++)
                        {
                            if (x + k < 0) continue;
                            if (x + k >= 79) continue;

                            for (int8_t l = -impact; l < impact + 1; l++)
                            {
                                if (y + l < 0) continue;
                                if (y + l >= 51) continue;

                                int dist = k * k + l * l;
                                if (dist <= impact * impact)
                                {
                                    if (rng->getFloat(0.0f, 1.0f) < 0.15f) continue;

                                    dig[x + k][y + l] = '*';
                                    digability[x + k][y + l] = rng->getFloat(0.8f, 1.0f);
                                }
                            }
                        }
                    }
                    else if (digability[x][y] < 0.2f)
                    {
                        flood_fill_freelist.push_back(XY{ x, y });
                        flood_fill_candidate.set(ixy);
                    }
                }
            }
        }
    }
}

void Level::minesweep()
{
    TCODRandom* rng = TCODRandom::getInstance();

    int dig_attempts = 20;
    int bomb_count = 10;
    for (int i = 0; i < dig_attempts; i++)
    {
        auto x = (int8_t)rng->getInt(5, WIDTH - 5);
        auto y = (int8_t)rng->getInt(5, HEIGHT - 5);
        flood_fill_freelist.push_back(XY{ x, y });
        flood_fill(bomb_count);
        if (i % 3 == 0) bomb_count--;
    }
}

void Level::connect()
{
    int index = 0;
    std::vector<graphs::NodeEntity> nodes;

    for (auto xy : exploded_bombs)
    {
        auto node = dig_plan.create_node();
        dig_plan.tag_node<XY>(node, xy.x, xy.y);
        nodes.push_back(node);
    }

    for (auto node1 : nodes)
    {
        for (auto node2 : nodes)
        {
            if (node1 == node2) continue;

            auto arrow = dig_plan.create_arrows(node1, node2);
            auto xy1 = dig_plan.get_tag<XY>(node1);
            auto xy2 = dig_plan.get_tag<XY>(node2);
            dig_plan.weigh_edge(arrow, xy1.distance(xy2));
        }
    }

    auto mst = dig_plan.get_minimum_spanning_tree();

    TCODRandom* rng = TCODRandom::getInstance();
    map = new TCODMap(WIDTH, HEIGHT);
    for (int i = 0; i < WIDTH; i++)
    {
        for (int j = 0; j < HEIGHT; j++)
        {
            bool ok = rng->getFloat(0.0f, 1.0f) > 0.13f;
            map->setProperties(i, j, ok, ok);
        }
    }

    auto d = TCOD_dijkstra_new_using_function(WIDTH, HEIGHT, [](int xFrom, int yFrom, int xTo, int yTo, void* user_data) -> float {
        Level* level = (Level*)user_data;
        if (level->map->isWalkable(xTo, yTo))
        {
            return (float)(std::abs(xFrom - xTo) + std::abs(yFrom - yTo));
        }
        else
        {
            return 0.0f;
        }
        }, this, 0.0f);

    for (auto edge : mst)
    {
        auto source = dig_plan.get_tag<XY>(dig_plan.get_source(edge));
        auto target = dig_plan.get_tag<XY>(dig_plan.get_target(edge));

        TCOD_dijkstra_compute(d, source.x, source.y);
        if (TCOD_dijkstra_path_set(d, target.x, target.y))
        {
            int x, y;
            while (TCOD_dijkstra_path_walk(d, &x, &y))
            {
                if (dig[x][y] != '.')
                {
                    digability[x][y] = 0.0f;
                    dig[x][y] = '.';
                }
            }
        }
    }

    TCOD_dijkstra_delete(d);
}

void Level::cellular_automata()
{
    // ovo ce da prodje ceo ekran
    for (int i = 0; i < WIDTH; i++)
    {
        for (int j = 0; j < HEIGHT; j++)
        {
            if (dig[i][j] == ' ') continue;

            int count_neighbors = 0;
            // ovo ce da prodje sve komsije
            for (int k = -1; k < 2; k++)
            {
                for (int l = -1; l < 2; l++)
                {
                    if (k == 0 && l == 0) continue;

                    int x = i + k;
                    int y = j + l;

                    if (x < 0) continue;
                    if (y < 0) continue;
                    if (x >= WIDTH) continue;
                    if (y >= HEIGHT) continue;

                    if (dig[x][y] != ' ')
                    {
                        count_neighbors++;
                    }
                }
            }

            if (count_neighbors == 0)
            {
                dig[i][j] = ' ';
            }
        }
    }
}

void Level::flood_fill_rooms(int start_x, int start_y, char current_room)
{
    TCODRandom* rng = TCODRandom::getInstance();

    flood_fill_freelist.clear();
    flood_fill_freelist.push_back(XY{ (int8_t)start_x, (int8_t)start_y });

    flood_fill_candidate.reset();
    flood_fill_visited.reset();

    while (!flood_fill_freelist.empty())
    {
        auto next = flood_fill_freelist.front();
        flood_fill_freelist.pop_front();

        rooms[next.x][next.y] = current_room;
        int index = (int)(current_room - '1');
        tiles[index].push_back(XY{ next.x, next.y });
        tiles_in_room[index]++;
        flood_fill_visited.set(TO_XY(next.x, next.y));

        for (int8_t i = -1; i < 2; i++)
        {
            if (next.x + i < 0) continue;
            if (next.x + i >= WIDTH - 1) continue;

            for (int8_t j = -1; j < 2; j++)
            {
                if (next.y + j < 0) continue;
                if (next.y + j >= HEIGHT - 1) continue;

                if (i == 0 && j == 0) continue;

                int8_t x = (int8_t)(next.x + i);
                int8_t y = (int8_t)(next.y + j);

                int ixy = TO_XY(x, y);

                if (flood_fill_candidate.test(ixy)) continue;

                if (!flood_fill_visited.test(ixy))
                {
                    if (dig[x][y] != ' ' && rooms[x][y] == ' ')
                    {
                        flood_fill_freelist.push_back(XY{ x, y });
                        flood_fill_candidate.set(ixy);
                    }
                }
            }
        }
    }
}

void Level::room_counting()
{
    tiles->clear();
    for (int i = 0; i < ROOM_COUNT; i++)
    {
        tiles_in_room[i] = 0;
    }

    for (int i = 0; i < WIDTH; i++)
    {
        for (int j = 0; j < HEIGHT; j++)
        {
            rooms[i][j] = ' ';
        }
    }

    char current_room = '1';
    for (int i = 0; i < WIDTH; i++)
    {
        for (int j = 0; j < HEIGHT; j++)
        {
            if (dig[i][j] != ' ' && rooms[i][j] == ' ')
            {
                flood_fill_rooms(i, j, current_room++);
                walkable.push_back(XY{ (int8_t)i, (int8_t)j });
            }
        }
    }

    for (int n = 0; n < ROOM_COUNT; n++)
    {
        if (tiles_in_room[n] < MIN_TILES_PER_ROOM)
        {
            for (int i = 0; i < WIDTH; i++)
            {
                for (int j = 0; j < HEIGHT; j++)
                {
                    if (rooms[i][j] == (char)(n + '1'))
                    {
                        dig[i][j] = ' ';
                        rooms[i][j] = ' ';
                    }
                }
            }

            tiles_in_room[n] = 0;
            tiles[n].clear();
        }
    }
}

void Level::force_connect()
{
    int index = 0;
    std::vector<graphs::NodeEntity> nodes;

    for (int i = 0; i < ROOM_COUNT; i++)
    {
        if (tiles_in_room[i] >= MIN_TILES_PER_ROOM)
        {
            auto xy = tiles[i][tiles_in_room[i] / 2];
            auto node = dig_plan.create_node();
            dig_plan.tag_node<XY>(node, xy.x, xy.y);
            nodes.push_back(node);
        }
    }

    if (nodes.size() <= 1) return;

    for (auto node1 : nodes)
    {
        for (auto node2 : nodes)
        {
            if (node1 == node2) continue;

            auto arrow = dig_plan.create_arrows(node1, node2);
            auto xy1 = dig_plan.get_tag<XY>(node1);
            auto xy2 = dig_plan.get_tag<XY>(node2);
            dig_plan.weigh_edge(arrow, xy1.distance(xy2));
        }
    }

    auto mst = dig_plan.get_minimum_spanning_tree();

    TCODRandom* rng = TCODRandom::getInstance();
    map = new TCODMap(WIDTH, HEIGHT);
    for (int i = 0; i < WIDTH; i++)
    {
        for (int j = 0; j < HEIGHT; j++)
        {
            map->setProperties(i, j, true, true);
        }
    }

    auto d = TCOD_dijkstra_new_using_function(WIDTH, HEIGHT, [](int xFrom, int yFrom, int xTo, int yTo, void* user_data) -> float {
        Level* level = (Level*)user_data;
        if (level->map->isWalkable(xTo, yTo))
        {
            return (float)(std::abs(xFrom - xTo) + std::abs(yFrom - yTo));
        }
        else
        {
            return 0.0f;
        }
        }, this, 1.41f);

    for (auto edge : mst)
    {
        auto source = dig_plan.get_tag<XY>(dig_plan.get_source(edge));
        auto target = dig_plan.get_tag<XY>(dig_plan.get_target(edge));

        TCOD_dijkstra_compute(d, source.x, source.y);
        if (TCOD_dijkstra_path_set(d, target.x, target.y))
        {
            int x, y;
            while (TCOD_dijkstra_path_walk(d, &x, &y))
            {
                if (dig[x][y] != '.')
                {
                    digability[x][y] = 0.0f;
                    dig[x][y] = '.';
                }
            }
        }
    }

    TCOD_dijkstra_delete(d);
}

void Level::flood_fill_regions()
{
    TCODRandom* rng = TCODRandom::getInstance();

    int regions_left = REGION_COUNT;
    int length = walkable.size() - 1;

    for (int i = 0; i < REGION_COUNT; i++)
    {
        region_tiles[i].clear();
    }

    for (int i = 0; i < WIDTH; i++)
    {
        for (int j = 0; j < HEIGHT; j++)
        {
            regions[i][j] = ' ';
        }
    }

    for (int i = 0; i < regions_left; i++)
    {
        XY xy{ 0, 0 };

        float current_distance = 20;

        int attempts = 0;

        while (true) {
            attempts++;
            if (attempts % 10 == 0)
            {
                current_distance -= 2;
                if (current_distance < 1) current_distance = 1;
            }

            xy = walkable[rng->getInt(0, length)];

            if (regions[xy.x][xy.y] != ' ') continue;

            bool okay = true;
            for (auto center : region_centers)
            {
                auto d = std::sqrt(center.distance(xy));
                if (d < current_distance) okay = false;
            }

            if (okay)
                break;
        }

        region_centers.push_back(xy);

        if (regions[xy.x][xy.y] == ' ')
        {
            int x = xy.x;
            int y = xy.y;

            int impact = rng->getInt(5, 7);

            // zumance
            for (int8_t k = -impact; k < impact + 1; k++)
            {
                if (x + k < 0) continue;
                if (x + k >= 79) continue;

                for (int8_t l = -impact; l < impact + 1; l++)
                {
                    if (y + l < 0) continue;
                    if (y + l >= 51) continue;

                    int dist = k * k + l * l;
                    if (dist <= impact * impact)
                    {
                        if (dig[x + k][y + l] != ' ' && dig[x + k][y + l] != '0')
                        {
                            regions[x + k][y + l] = '1' + i;
                            region_tiles[i].push_back(XY{ (int8_t)(x + k), (int8_t)(y + l) });
                        }
                    }
                }
            }
        }
    }
}

void Level::update_map_visibility()
{
    for (int i = 0; i < WIDTH; i++)
    {
        for (int j = 0; j < HEIGHT; j++)
        {
            map->setProperties(i, j, dig[i][j] != ' ', dig[i][j] != ' ');
        }
    }
}

void Level::generate()
{
    init();

    gradient();
    minesweep();
    cellular_automata();

    room_counting();
    connect();
    force_connect();

    room_counting();
    walkable = tiles[0];

    flood_fill_regions();
    update_map_visibility();
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
        queues.insert({ std::string(key), queue });
    }
}

PeopleMapping LevelCreationSystem::generate_people_graph() {
    PeopleMapping map;

    map.graph = std::shared_ptr<graphs::Graph>(new graphs::Graph());
    auto& places = map.places;
    auto& people = map.people;
    auto& graph = map.graph;

    TCODRandom* rng = TCODRandom::getInstance();
    std::unordered_set<int> used_places;

    Residents residents[REGION_COUNT] {};
    for (int i = 0; i < REGION_COUNT; i++)
    {
        places[i] = graph->create_node();
        graph->label_node(places[i], "place #" + std::to_string(i + 1));
        graph->tag_node<Place>(places[i], i);        
    }

    for (int i = 0; i < PEOPLE_COUNT; i++)
    {
        people[i] = graph->create_node();
        graph->tag_node<Person>(people[i], i);
        graph->label_node(people[i], "person #" + std::to_string(i + 1));

        int p = rng->getInt(0, REGION_COUNT - 1);
        auto lives_in_arrow = graph->create_arrow(people[i], places[p]);
        graph->label_edge(lives_in_arrow, "lives in");
        residents[p].living.push_back(i);
        graph->tag_edge<LivesIn>(lives_in_arrow);
        used_places.insert(p);

        p = rng->getInt(0, REGION_COUNT - 1);
        auto works_in_arrow = graph->create_arrow(people[i], places[p]);
        graph->label_edge(works_in_arrow, "works in");
        residents[p].working.push_back(i);
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
            residents[j].visits.push_back(current_person % PEOPLE_COUNT);
            current_person++;
        }
    }

    current_person = 0;
    for (int j = 0; j < REGION_COUNT; j++)
    {
        if (rng->getInt(0, 100) > 95) continue;
        int ps = std::max(PEOPLE_COUNT - 1, 2 + rng->getInt(0, PEOPLE_COUNT - 1));

        for (int i = 0; i < ps; i++)
        {
            if (rng->getInt(0, 100) > 30) continue;
            auto uses_arrow = graph->create_arrow(people[current_person % PEOPLE_COUNT], places[j]);
            graph->label_edge(uses_arrow, "visits");
            graph->tag_edge<Visits>(uses_arrow);
            residents[j].visits.push_back((current_person * rng->getInt(1, 10)) % PEOPLE_COUNT);
            current_person++;
        }
    }

    std::unordered_map<std::string, PlaceWeightQueue> queues;
    auto places_yaml = AccessYAML::load("data/lists/places.yaml");

    populate_queues(places_yaml, queues);

    for (int i = 0; i < REGION_COUNT; i++)
    {
        const int w = residents[i].working.size();
        const int l = residents[i].living.size();
        const int v = residents[i].visits.size();

        auto kind = PlaceKind::SomeVisits;
        if (w == 0 && v > 0)
        {
            kind = PlaceKind::NoWorkOnlyVisit;
        }
        else if (w > 0 && v == 0)
        {
            kind = PlaceKind::FewWorkNoVisits;
        }
        else if (w > 0 && v > 0 && v > 1.25f * w)
        {
            kind = PlaceKind::FewWorkManyVisits;
        }
        else if (w > 0 && v > 0 && w > 1.25f * v)
        {
            kind = PlaceKind::ManyWorkFewVisits;
        }
        else if (w > 0 && v > 0)
        {
            kind = PlaceKind::SomeWorkSomeVisit;
        }

        auto place_kind = std::string(get_place_kind(kind));
        if (queues[place_kind].empty())
        {
            populate_queues(places_yaml, queues);
        }

        auto place = queues[place_kind].top();
        queues[place_kind].pop();
        auto value = std::get<0>(place);
        auto prio = std::get<1>(place);
        
        printf("%d) W%d L%d V%d = %s (%s)\n", i, w, l, v, place_kind.c_str(), value.c_str());
    }
    
    printf("-------------------");
    //graph->print();

    return map;
}

void LevelCreationSystem::generate()
{
    std::unordered_set<int> used_spaces;

    TCODRandom* rng = TCODRandom::getInstance();

    AccessWorld_UseUnique<Level>::access_unique().generate();
    auto& people_mapping = AccessWorld_UseUnique<PeopleMapping>::access_unique();
    people_mapping.graph.reset();

    auto queried = AccessWorld_QueryAllEntitiesWith<Person>::query();
    AccessWorld_ModifyWorld::destroy_entities(queried.begin(), queried.end());    

    people_mapping = generate_people_graph();

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
                
                auto person = AccessWorld_ModifyWorld::create_entity();
                AccessWorld_ModifyEntity::add_component<Person>(person, person_id);
                AccessWorld_ModifyEntity::add_component<Sex>(person, sex);
                AccessWorld_ModifyEntity::add_component<Health>(person, 100, 100);
                AccessWorld_ModifyEntity::add_component<ActionPoints>(person, 0);
                AccessWorld_ModifyEntity::add_component<Speed>(person, rng->getInt(80, 110));

                int letter_index = rng->getInt(0, letters.size() - 1);
                char c = letters[letter_index];
                std::string s_low(1, c);
                letters.erase(letters.begin() + letter_index);

                YAML::Node name_list = (sex == Sex::Female ? female_name_list : male_name_list)[s_low];
                int name_index = rng->getInt(0, name_list.size() - 1);
                auto name = name_list[name_index].as<std::string>();
                name_list.remove(name_index);

                std::string s_high(1, c - 32);
                AccessWorld_ModifyEntity::add_component<Symbol>(person, s_high);
                AccessWorld_ModifyEntity::add_component<Name>(person, name);
                AccessWorld_ModifyEntity::add_component<AIPlayer>(person);
                AccessWorld_ModifyEntity::add_component<WorldPosition>(person, (int)tile.x, (int)tile.y);
            }
        }
    }
}

void LevelCreationSystem::activate()
{
    generate();
    AccessEvents_Emit<LevelCreationEvent>::emit_event();
}

void LevelCreationSystem::react_to_event(KeyEvent& signal)
{
    if (signal.key == KeyCode::KEY_SPACE)
    {
        generate();
        AccessEvents_Emit<LevelCreationEvent>::emit_event();
    }
}


void Debug_RoomLevelRenderSystem::activate()
{
    if (mode == Debug_RenderMode::Off) return;

    const auto& level = access_unique();

    if (mode == Debug_RenderMode::RoomNumbers)
    {
        for (int i = 0; i < 80; i++)
        {
            for (int j = 0; j < 52; j++)
            {
                if (level.rooms[i][j] != ' ')
                {
                    std::string s(1, level.rooms[i][j]);
                    ch({ i, j }, s);
                }
            }
        }
    }
    else if (mode == Debug_RenderMode::Regions)
    {
        for (int i = 0; i < 80; i++)
        {
            for (int j = 0; j < 52; j++)
            {
                if (level.regions[i][j] != ' ')
                {
                    std::string s(1, level.regions[i][j]);
                    ch({ i, j }, s);
                }
            }
        }
    }
}

void Debug_RoomLevelRenderSystem::react_to_event(KeyEvent& signal)
{
    if (signal.key == KeyCode::KEY_TAB)
    {
        mode = (Debug_RenderMode)(((int)mode + 1) % Debug_RenderMode::COUNT);
    }
}


void LevelRenderSystem::activate()
{    
    tick++;

    TCODRandom* rng = TCODRandom::getInstance();
    auto& level = AccessWorld_UseUnique<Level>::access_unique();
    const auto& colors = AccessWorld_UseUnique<Colors>::access_unique();

    const auto player_entity = AccessWorld_QueryAllEntitiesWith<Player>::query().front();
    const auto& world_pos = AccessWorld_QueryComponent<WorldPosition>::get_component(player_entity);
    const auto& sight = AccessWorld_QueryComponent<Sight>::get_component(player_entity);

    const auto xy = XY{ (int8_t)world_pos.x, (int8_t)world_pos.y };
    const auto rad = (float)sight.radius;
    const auto radius = (float)sight.radius * 2.0f;

    for (int i = 0; i < 80; i++)
    {
        for (int j = 0; j < 52; j++)
        {
            if (level.map->isInFov(i, j))
            {
                const auto ij = XY{ (int8_t)i, (int8_t)j };
                const auto dist = xy.distance(ij);

                float dist_factor = 1.0f;
                if (dist >= sight.radius * 0.9f)
                {
                    dist_factor = colors.visible_shift_very_far;
                }
                else if (dist >= sight.radius * 0.75f)
                {
                    dist_factor = colors.visible_shift_far;
                }
                else if (dist >= sight.radius * 0.5f)
                {
                    dist_factor = colors.visible_shift_mid;
                }

                if (level.dig[i][j] == ' ')
                {
                    AccessConsole::fg({ i, j }, HSL(colors.visible_hue, colors.visible_sat, dist_factor));
                    AccessConsole::ch({ i, j }, "#");
                }
                else if (level.dig[i][j] == '*')
                {
                    auto time_factor = std::sin((i + j) * colors.shimmer_stripe_width + tick * colors.shimmer_stripe_speed);
                    bg({ i, j }, HSL(colors.shimmer_hue + time_factor * colors.shimmer_stripe_strength, 1.0f,
                        rng->getFloat(0.95f, 1.0f) * (rad - xy.distance(ij)) / radius));
                    AccessConsole::fg({ i, j }, HSL(255.0f, 0.3f, 2 * (radius - xy.distance(ij)) / rad));
                    level.memory[i][j] = '.';
                    ch({ i, j }, ".");
                }
                else
                {
                    AccessConsole::fg({ i, j }, HSL(colors.visible_hue, colors.visible_sat, dist_factor));
                    std::string s(1, level.dig[i][j]);
                    level.memory[i][j] = '.';
                    AccessConsole::ch({ i, j }, s);
                }
            }
            else
            {
                AccessConsole::fg({ i, j }, HSL(colors.memory_hue, colors.memory_sat, colors.memory_lit));

                std::string s(1, level.dig[i][j]);
                s[0] = level.memory[i][j];
                AccessConsole::ch({ i, j }, s);
            }
        }
    }
}
