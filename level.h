#pragma once

#include <deque>
#include <unordered_set>
#include <bitset>
#include <libtcod.hpp>
#include <queue>

#include "config.h"
#include "common.h"

#include "graphs.h"

struct Level
{
    TCODMap* map;
    
    float digability[WIDTH][HEIGHT] { 0.0f, };
    char dig[WIDTH][HEIGHT]{ ' ', };
    char memory[WIDTH][HEIGHT]{ ' ', };

    char rooms[WIDTH][HEIGHT]{ ' ', };
    char regions[WIDTH][HEIGHT]{ ' ', };

    std::vector<XY> region_centers;
    std::vector<XY> region_tiles[REGION_COUNT];

    int tiles_in_room[ROOM_COUNT] { 0, };
    std::vector<XY> tiles[ROOM_COUNT];

    std::vector<XY> walkable;
    std::bitset<WIDTH * HEIGHT> flood_fill_visited;
    std::bitset<WIDTH * HEIGHT> flood_fill_candidate;
    std::bitset<WIDTH * HEIGHT> bombs;
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

struct PeopleMapping;
struct Person;

struct LevelCreationEvent {};

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
    PeopleMapping generate_people_graph();
    void generate();
    void activate() override;
    void react_to_event(KeyEvent& signal) override;
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

enum Debug_RenderMode
{
    Off = 0,
    RoomNumbers,
    Regions,
    COUNT
};

struct Debug_RoomLevelRenderSystem
    : public RuntimeSystem
    , public AccessConsole
    , public AccessEvents_Listen<KeyEvent>
    , public AccessWorld_UseUnique<Level>
{
    Debug_RenderMode mode = Debug_RenderMode::Off;

    void activate() override;

    void react_to_event(KeyEvent& signal) override;
};

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
