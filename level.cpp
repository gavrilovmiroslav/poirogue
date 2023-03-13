#include "level.h"

#include "config.h"
#include "common.h"

#include <unordered_map>
#include <yaml-cpp/yaml.h>

void Level::init()
{
    walkable.clear();

    exploded_bombs.clear();

    const auto& colors = AccessWorld_UseUnique<Colors>::access_unique();
    TCODRandom* rng = TCODRandom::getInstance();

    for (int i = 0; i < MAP_WIDTH; i++)
    {
        for (int j = 0; j < MAP_HEIGHT; j++)
        {
            dig[i][j] = ' ';
            rooms[i][j] = ' ';
            hues[i][j] = colors.visible_hue;
            sats[i][j] = colors.visible_sat;
            vals[i][j] = 0.75f;
            digability[i][j] = rng->getFloat(0.0f, 1.0f, 0.5f);
        }
    }
}

void Level::gradient()
{
    TCODRandom* rng = TCODRandom::getInstance();

    float f = 1.0f;
    float radius = 10.0f;
    for (int i = 0; i < 450; i++)
    {
        f *= 0.9991f;
        int x = rng->getInt(5, MAP_WIDTH - 5);
        int y = rng->getInt(5, MAP_HEIGHT - 5);

        for (int i = -(int)radius; i < (int)radius; i++)
        {
            for (int j = -(int)radius; j < (int)radius; j++)
            {
                if (x + i < 0 || x + i > MAP_WIDTH - 1) continue;
                if (y + j < 0 || y + j > MAP_HEIGHT - 1) continue;

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
        int8_t x = rng->getInt(5, MAP_WIDTH - 5);
        int8_t y = rng->getInt(5, MAP_HEIGHT - 5);
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
            if (next.x + i >= MAP_WIDTH - 1) continue;

            for (int8_t j = -1; j < 2; j++)
            {
                if (next.y + j < 0) continue;
                if (next.y + j >= MAP_HEIGHT - 1) continue;

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
                            if (x + k >= MAP_WIDTH - 1) continue;

                            for (int8_t l = -impact; l < impact + 1; l++)
                            {
                                if (y + l < 0) continue;
                                if (y + l >= MAP_HEIGHT - 1) continue;

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
        auto x = (int8_t)rng->getInt(5, MAP_WIDTH - 5);
        auto y = (int8_t)rng->getInt(5, MAP_HEIGHT - 5);
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
    map = new TCODMap(MAP_WIDTH, MAP_HEIGHT);
    for (int i = 0; i < MAP_WIDTH; i++)
    {
        for (int j = 0; j < MAP_HEIGHT; j++)
        {
            bool ok = rng->getFloat(0.0f, 1.0f) > 0.13f;
            map->setProperties(i, j, ok, ok);
        }
    }

    auto d = TCOD_dijkstra_new_using_function(MAP_WIDTH, MAP_HEIGHT, [](int xFrom, int yFrom, int xTo, int yTo, void* user_data) -> float {
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
    for (int i = 0; i < MAP_WIDTH; i++)
    {
        for (int j = 0; j < MAP_HEIGHT; j++)
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
                    if (x >= MAP_WIDTH) continue;
                    if (y >= MAP_HEIGHT) continue;

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
        tiles[index].push_back({ (int)next.x, (int)next.y });
        tiles_in_room[index]++;
        flood_fill_visited.set(TO_XY(next.x, next.y));

        for (int8_t i = -1; i < 2; i++)
        {
            if (next.x + i < 0) continue;
            if (next.x + i >= MAP_WIDTH - 1) continue;

            for (int8_t j = -1; j < 2; j++)
            {
                if (next.y + j < 0) continue;
                if (next.y + j >= MAP_HEIGHT - 1) continue;

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

    for (int i = 0; i < MAP_WIDTH; i++)
    {
        for (int j = 0; j < MAP_HEIGHT; j++)
        {
            rooms[i][j] = ' ';
        }
    }

    char current_room = '1';
    for (int i = 0; i < MAP_WIDTH; i++)
    {
        for (int j = 0; j < MAP_HEIGHT; j++)
        {
            if (dig[i][j] != ' ' && rooms[i][j] == ' ')
            {
                flood_fill_rooms(i, j, current_room++);
                walkable.push_back({ i, j });
            }
        }
    }

    for (int n = 0; n < ROOM_COUNT; n++)
    {
        if (tiles_in_room[n] < MIN_TILES_PER_ROOM)
        {
            for (int i = 0; i < MAP_WIDTH; i++)
            {
                for (int j = 0; j < MAP_HEIGHT; j++)
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
            dig_plan.tag_node<WorldPosition>(node, xy.x, xy.y);
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
            auto xy1 = dig_plan.get_tag<WorldPosition>(node1);
            auto xy2 = dig_plan.get_tag<WorldPosition>(node2);
            dig_plan.weigh_edge(arrow, xy1.distance(xy2));
        }
    }

    auto mst = dig_plan.get_minimum_spanning_tree();

    TCODRandom* rng = TCODRandom::getInstance();
    map = new TCODMap(MAP_WIDTH, MAP_HEIGHT);
    for (int i = 0; i < MAP_WIDTH; i++)
    {
        for (int j = 0; j < MAP_HEIGHT; j++)
        {
            map->setProperties(i, j, true, true);
        }
    }

    auto d = TCOD_dijkstra_new_using_function(MAP_WIDTH, MAP_HEIGHT, 
        [](int xFrom, int yFrom, int xTo, int yTo, void* user_data) -> float {
        
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
        auto source = dig_plan.get_tag<WorldPosition>(dig_plan.get_source(edge));
        auto target = dig_plan.get_tag<WorldPosition>(dig_plan.get_target(edge));

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

    for (int i = 0; i < MAP_WIDTH; i++)
    {
        for (int j = 0; j < MAP_HEIGHT; j++)
        {
            regions[i][j] = ' ';
        }
    }

    for (int i = 0; i < regions_left; i++)
    {
        WorldPosition xy{ 0, 0 };

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
                if (x + k >= MAP_WIDTH - 1) continue;

                for (int8_t l = -impact; l < impact + 1; l++)
                {
                    if (y + l < 0) continue;
                    if (y + l >= MAP_HEIGHT - 1) continue;

                    int dist = k * k + l * l;
                    if (dist <= impact * impact)
                    {
                        if (dig[x + k][y + l] != ' ' && dig[x + k][y + l] != '0')
                        {
                            regions[x + k][y + l] = '1' + i;
                            region_tiles[i].push_back({ x + k, y + l });
                        }
                    }
                }
            }
        }
    }
}

void Level::update_map_visibility()
{
    for (int i = 0; i < MAP_WIDTH; i++)
    {
        for (int j = 0; j < MAP_HEIGHT; j++)
        {
            memory[i][j] = ' ';
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

void LevelCreationSystem::activate()
{   
    auto all_in_world = AccessWorld_QueryAllEntitiesWith<WorldPosition>().query();
    AccessWorld_ModifyWorld::destroy_entities(all_in_world.begin(), all_in_world.end());

    auto& calendar = AccessWorld_UseUnique<Calendar>::access_unique();
    calendar.day = 1;
    calendar.hour = 1;
    calendar.minute = 1;

    AccessWorld_UseUnique<Level>::access_unique().generate();

    auto& pm = AccessWorld_UseUnique<PeopleMapping>::access_unique();

    for (auto& crafting : pipeline)
    {
        crafting->execute_crafting();
    }

    AccessEvents_Emit<LevelCreationEvent>::emit_event();
}

void LevelCreationSystem::react_to_event(KeyEvent& signal)
{
    if (signal.key == KeyCode::KEY_F3)
    {
        activate();
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

    const auto rad = (float)sight.radius;
    const auto radius = (float)sight.radius * 2.0f;

    for (int i = 0; i < MAP_WIDTH; i++)
    {
        for (int j = 0; j < MAP_HEIGHT; j++)
        {
            const auto scr = ScreenPosition{ i, j };
            if (level.map->isInFov(i, j))
            {
                const auto ij = WorldPosition{ i, j };
                const auto dist = world_pos.distance(ij);

                float dist_factor = 1.0f;
                if (dist >= sight.radius * 0.9f)
                {
                    dist_factor = 0.25f;
                }
                else if (dist >= sight.radius * 0.75f)
                {
                    dist_factor = 0.45f;
                }
                else if (dist >= sight.radius * 0.5f)
                {
                    dist_factor = 0.6f;
                }

                if (level.dig[i][j] == ' ')
                {
                    level.memory[i][j] = level.dig[i][j];
                    AccessConsole::fg(scr, HSL(level.hues[i][j], level.sats[i][j], dist_factor * level.vals[i][j]));
                    AccessConsole::ch(scr, "#");
                    level.memory[i][j] = '#';
                }
                else if (level.dig[i][j] == '*')
                {
                    auto time_factor = std::sin((i + j) * colors.shimmer_stripe_width + tick * colors.shimmer_stripe_speed);
                    bg(scr, HSL(level.hues[i][j] + time_factor * colors.shimmer_stripe_strength, 1.0f,
                        rng->getFloat(0.95f, 1.0f) * (rad - world_pos.distance(ij)) / radius));
                    AccessConsole::fg(scr, HSL(255.0f, 0.3f, 2 * (radius - world_pos.distance(ij)) / rad));
                    level.memory[i][j] = '.';
                    ch(scr, ".");
                }
                else
                {
                    AccessConsole::fg(scr, HSL(level.hues[i][j], colors.visible_sat, dist_factor));
                    std::string s(1, level.dig[i][j]);
                    level.memory[i][j] = level.dig[i][j];
                    AccessConsole::ch(scr, s);
                }
            }
            else
            {
                AccessConsole::fg(scr, HSL(colors.memory_hue, colors.memory_sat, colors.memory_lit));

                std::string s(1, level.dig[i][j]);
                s[0] = level.memory[i][j];
                AccessConsole::ch(scr, s);
            }
        }
    }
}
