#include "engine.h"
#include "utils.h"
#include "graphs.h"
#include "map.h"

#include <unordered_map>

#undef main


struct LevelCreationSystem
    : public OneOffSystem
    , public AccessWorld_Unique<Level>
    , public AccessEvents_Listen<KeyEvent>
{
    void activate() override
    {
        access_unique().generate();
    }

    void react_to_event(KeyEvent& signal)
    {
        if (signal.key == KeyCode::KEY_SPACE)
        {
            access_unique().generate();
        }
    }
};

struct LevelRenderSystem
    : public RuntimeSystem    
    , public AccessConsole
    , public AccessWorld_Unique<Level>
{
    void activate() override
    {
        TCODRandom* rng = TCODRandom::getInstance();
        const auto& level = access_unique();
        
        for (int i = 0; i < 80; i++)
        {
            for (int j = 0; j < 52; j++)
            {
                float f = level.floor[i][j] * 0.5f;
                if (f > 0.1f && level.dig[i][j] != ' ')
                {
                    bg({ i, j }, HSL(rng->getFloat(160.0f, 190.0f), f, rng->getFloat(0.5f, 0.85f)));
                }
                else
                {
                    bg({ i, j }, RGB(0, 0, 0));
                }

                fg({ i, j }, RGB(128, 128, 128));
                
                std::string s(1, level.dig[i][j]);
                ch({ i, j }, s);
            }
        }
    }
};

struct Person {
    std::string name;
};

struct Place {};

int main(int argc, char* argv[])
{
    srand(1);
    constexpr const int PEOPLE_COUNT = 5;
    constexpr const int PLACE_COUNT = 10;

    graphs::NodeEntity people[PEOPLE_COUNT];
    graphs::NodeEntity places[PLACE_COUNT];

    graphs::Graph graph;
    std::unordered_set<int> used_places;

    for (int i = 0; i < PLACE_COUNT; i++)
    {
        places[i] = graph.create_node();
        graph.label_node(places[i], "place #" + std::to_string(i));
        graph.tag_node<Place>(places[i]);        
    }

    for (int i = 0; i < PEOPLE_COUNT; i++)
    {
        people[i] = graph.create_node();
        graph.label_node(people[i], "person #" + std::to_string(i));

        int p = rand() % (PLACE_COUNT);
        auto lives_in_arrow = graph.create_arrow(people[i], places[p]);
        graph.label_edge(lives_in_arrow, "lives in");
        used_places.insert(p);
        
        p = rand() % (PLACE_COUNT);
        auto works_in_arrow = graph.create_arrow(people[i], places[p]);
        graph.label_edge(works_in_arrow, "works in");
        used_places.insert(p);
    }

    int current_person = 0;
    for (int j = 0; j < PLACE_COUNT; j++)
    {
        if (used_places.count(j) > 0 || rand() % 100 > 85) continue;
        int ps = std::max(PEOPLE_COUNT - 1, 2 + rand() % PEOPLE_COUNT);

        for (int i = 0; i < ps; i++)
        {                        
            auto uses_arrow = graph.create_arrow(people[current_person % PEOPLE_COUNT], places[j]);
            graph.label_edge(uses_arrow, "visits");
            current_person++;
        }        
    }

    graph.print();
    
    PoirogueEngine engine;
    engine.add_one_off_system<LevelCreationSystem>();
    engine.add_runtime_system<LevelRenderSystem>();
    engine.restart_game();

    while (engine) {
        engine.start_frame();
        engine.poll_events();
        engine.run_systems();
        engine.end_frame();
    }
}
