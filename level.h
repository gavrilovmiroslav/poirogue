#pragma once

#include <deque>
#include <unordered_set>
#include <bitset>
#include <libtcod.hpp>

#include "config.h"
#include "common.h"

#include "graphs.h"

struct Level
{
    TCODMap* map;
    
    float digability[WIDTH][HEIGHT] { 0.0f, };
    char dig[WIDTH][HEIGHT]{ ' ', };

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
};

struct PeopleMapping;
struct Person;

struct LevelCreationEvent {};

struct LevelCreationSystem
    : public OneOffSystem
    , public AccessWorld_Unique<Level>
    , public AccessWorld_Unique<PeopleMapping>
    , public AccessWorld_QueryByEntity<Person>
    , public AccessEvents_Listen<KeyEvent>
    , public AccessEvents_Emit<LevelCreationEvent>
    , public AccessWorld_ModifyWorld
    , public AccessWorld_ModifyEntity
{
    void generate();
    void activate() override;
    void react_to_event(KeyEvent& signal) override;
};


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
    , public AccessWorld_Unique<Level>
{
    Debug_RenderMode mode = Debug_RenderMode::Off;

    void activate() override;

    void react_to_event(KeyEvent& signal) override;
};

struct LevelRenderSystem
    : public RuntimeSystem    
    , public AccessConsole
    , public AccessWorld_Unique<Level>
{
    void activate() override;
};

struct ShimmerRenderSystem
    : public RuntimeSystem
    , public AccessConsole
    , public AccessWorld_Unique<Level>    
{
    int tick;

    void activate() override;
};