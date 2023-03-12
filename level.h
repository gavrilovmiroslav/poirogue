#pragma once

#include <deque>
#include <unordered_set>
#include <bitset>
#include <libtcod.hpp>
#include <queue>
#include <functional>

#include "config.h"
#include "common.h"

#include "graphs.h"

struct PeopleMapping;
struct Person;

struct LevelCreationEvent {};

struct Level
    : public AccessWorld_ModifyWorld
    , public AccessWorld_ModifyEntity
    , public AccessWorld_UseUnique<Colors>
{
    TCODMap* map;

    float digability[MAP_WIDTH][MAP_HEIGHT] { 0.0f, };
    char dig[MAP_WIDTH][MAP_HEIGHT]{ ' ', };
    char memory[MAP_WIDTH][MAP_HEIGHT]{ ' ', };

    float hues[MAP_WIDTH][MAP_HEIGHT]{ 0.0f };
    char rooms[MAP_WIDTH][MAP_HEIGHT]{ ' ', };
    char regions[MAP_WIDTH][MAP_HEIGHT]{ ' ', };

    std::vector<WorldPosition> region_centers;
    std::vector<WorldPosition> region_tiles[REGION_COUNT];

    int tiles_in_room[ROOM_COUNT] { 0, };
    std::vector<WorldPosition> tiles[ROOM_COUNT];

    std::vector<WorldPosition> walkable;
    std::bitset<MAP_WIDTH * MAP_HEIGHT> flood_fill_visited;
    std::bitset<MAP_WIDTH * MAP_HEIGHT> flood_fill_candidate;
    std::bitset<MAP_WIDTH * MAP_HEIGHT> bombs;
    std::vector<XY> exploded_bombs;
    graphs::Graph dig_plan;

    std::deque<XY> flood_fill_freelist;

    Level() {}

    void init();
    void gradient();
    void flood_fill(int bomb_count);
    void minesweep();
    void connect();
    void cellular_automata();
    void flood_fill_rooms(int start_x, int start_y, char current_room);
    void room_counting();
    void force_connect();
    void flood_fill_regions();
    void generate();
    void update_map_visibility();
};

struct LevelCreationSystem
    : public OneOffSystem
    , public AccessWorld_UseUnique<Level>
    , public AccessWorld_UseUnique<PeopleMapping>
    , public AccessWorld_QueryAllEntitiesWith<Person>
    , public AccessEvents_Listen<KeyEvent>
    , public AccessEvents_Emit<LevelCreationEvent>
    , public AccessWorld_ModifyWorld
    , public AccessWorld_ModifyEntity
    , public AccessYAML
{    
    template<typename T>
    std::shared_ptr<T> add_pipeline()
    {
        std::shared_ptr<T> ptr{ new T };
        pipeline.push_back(ptr);
        return ptr;
    }

    void activate() override;
    void react_to_event(KeyEvent& signal) override;

private:
    std::vector<std::shared_ptr<CraftingPipeline>> pipeline;
};

using PlaceWeight = std::tuple<std::string, int>;

struct PlaceWeightSort
{
    bool operator() (const PlaceWeight l, const PlaceWeight r)
    {
        auto lv = std::get<1>(l);
        auto rv = std::get<1>(r);
        return lv < rv;
    }
};

using PlaceWeightQueue = std::priority_queue<PlaceWeight, std::vector<PlaceWeight>, PlaceWeightSort>;

struct LevelRenderSystem
    : public RuntimeSystem
    , public AccessConsole
    , public AccessYAML
    , public AccessWorld_QueryComponent<WorldPosition>
    , public AccessWorld_QueryComponent<Sight>
    , public AccessWorld_UseUnique<Level>
    , public AccessWorld_UseUnique<Colors>
    , public AccessWorld_QueryAllEntitiesWith<Player>
{
    int tick;

    void activate() override;
};
