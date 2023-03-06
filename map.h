#pragma once

#include <deque>
#include <unordered_set>
#include <bitset>
#include <libtcod.hpp>

#include "graphs.h"

#define TO_XY(x, y) ((int)x + 80l * (int)y)

struct Room
{
    int index;
    int x1, y1, x2, y2;

    Room(int index, int x1, int y1, int x2, int y2)
        : index{ index }
        , x1 { x1 }
        , y1{ y1 }
        , x2{ x2 }
        , y2{ y2 }
    {}

    inline int center_x()
    {
        return (x1 + x2) / 2;
    }

    inline int center_y()
    {
        return (y1 + y2) / 2;
    }
};

struct XY 
{
    int8_t x, y;

    inline float distance(XY& xy2)
    {
        float dx = ((float)x - (float)xy2.x);
        float dy = ((float)y - (float)xy2.y);
        return  dx * dx + dy * dy;
    }
};

struct Level : public ITCODBspCallback
             , public ITCODPathCallback
{
    TCODMap* map;
    float floor[80][52] { 0.0f, };
    char dig[80][52]{ ' ', };
    std::vector<Room> rooms;

    std::bitset<80 * 52> flood_fill_visited;
    std::bitset<80 * 52> flood_fill_candidate;
    std::bitset<80 * 52> bombs;
    std::vector<XY> exploded_bombs;
    graphs::Graph dig_plan;

    std::deque<XY> flood_fill_freelist;
    
    static int room_index;

    Level() {}

    void gradient()
    {
        TCODRandom* rng = TCODRandom::getInstance();

        for (int i = 0; i < 80; i++)
        {
            for (int j = 0; j < 52; j++)
            {
                dig[i][j] = ' ';
                floor[i][j] = rng->getFloat(0.0f, 1.0f, 0.5f);
            }
        }

        float f = 1.0f;
        float radius = 10.0f;
        for (int i = 0; i < 450; i++)
        {
            f *= 0.9991f;
            int x = rng->getInt(5, 75);
            int y = rng->getInt(5, 45);

            for (int i = -(int)radius; i < (int)radius; i++)
            {
                for (int j = -(int)radius; j < (int)radius; j++)
                {
                    if (x + i < 0 || x + i > 79) continue;
                    if (y + j < 0 || y + j > 51) continue;

                    floor[x + i][y + j] *= f;
                }
            }

            radius *= 0.995f;
            if (radius < 1.0f)
            {
                break;
            }
        }
    }

    void flood_fill(int bomb_count)
    {
        TCODRandom* rng = TCODRandom::getInstance();

        bombs.reset();
        for (int i = 0; i < bomb_count; i++)
        {
            int8_t x = rng->getInt(5, 75);
            int8_t y = rng->getInt(5, 45);
            bombs.set(TO_XY(x, y));
        }

        flood_fill_candidate.reset();
        flood_fill_visited.reset();

        while (!flood_fill_freelist.empty())
        {
            auto next = flood_fill_freelist.front();
            flood_fill_freelist.pop_front();

            dig[next.x][next.y] = '.';
            floor[next.x][next.y] = 0.0f;
            flood_fill_visited.set(TO_XY(next.x, next.y));

            for (int8_t i = -1; i < 2; i++)
            {
                if (next.x + i < 0) continue;
                if (next.x + i >= 79) continue;

                for (int8_t j = -1; j < 2; j++)
                {
                    if (next.y + j < 0) continue;
                    if (next.y + j >= 51) continue;
                    
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
                                    if (dist <= 3 * 3)
                                    {
                                        if (rng->getFloat(0.0f, 1.0f) < 0.15f) continue;

                                        dig[x + k][y + l] = '*';
                                        floor[x + k][y + l] = rng->getFloat(0.8f, 1.0f);
                                    }
                                }
                            }
                        }
                        else if (floor[x][y] < 0.2f)
                        {
                            flood_fill_freelist.push_back(XY{ x, y });
                            flood_fill_candidate.set(ixy);
                        }
                    }
                }
            }
        }
    }

    void minesweep()
    {
        TCODRandom* rng = TCODRandom::getInstance();

        int dig_attempts = 20;
        int bomb_count = 10;
        for (int i = 0; i < dig_attempts; i++)
        {
            auto x = (int8_t)rng->getInt(5, 75);
            auto y = (int8_t)rng->getInt(5, 47);
            flood_fill_freelist.push_back(XY{ x, y });
            flood_fill(bomb_count);
            if (i % 3 == 0) bomb_count--;
        }
    }

    void connect()
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
        map = new TCODMap(80, 52);
        for (int i = 0; i < 80; i++)
        {
            for (int j = 0; j < 52; j++)
            {
                bool ok = rng->getFloat(0.0f, 1.0f) > 0.33f;
                map->setProperties(i, j, ok, ok);
            }
        }

        auto d = TCOD_dijkstra_new_using_function(80, 52, [](int xFrom, int yFrom, int xTo, int yTo, void* user_data) -> float {
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
                        floor[x][y] = 0.0f;
                        dig[x][y] = 'x';
                    }
                }
            }
        }

        TCOD_dijkstra_delete(d);
    }

    void generate()
    {
        exploded_bombs.clear();
        rooms.clear();
        room_index = 0;

        gradient();
        minesweep();
        connect();
    }

    float getWalkCost(int xFrom, int yFrom, int xTo, int yTo, void* userData) const override
    {
        return 0.0f;
    }

    void evolve_cave(int x1, int y1, int x2, int y2, int index = -1)
    {
        if (x2 < x1) std::swap(x1, x2);
        if (y2 < y1) std::swap(y1, y2);

        for (int tilex = x1; tilex <= x2; tilex++) {
            for (int tiley = y1; tiley <= y2; tiley++) {
                floor[tilex][tiley] *= 0.5f;
            }
        }
    }

    bool visitNode(TCODBsp* node, void* user_data)
    {        
        TCODRandom* rng = TCODRandom::getInstance();
        if (node->level > 5 && rng->getInt(0, 100) > 15)
        {
            int w = rng->getInt(1, node->w);
            int h = rng->getInt(1, node->h);
            int x = rng->getInt(node->x, node->x + node->w - w);
            int y = rng->getInt(node->y, node->y + node->h - h);

            evolve_cave(x, y, x + w, y + h, room_index++);

            return true;
        }

        return false;
    }
};

