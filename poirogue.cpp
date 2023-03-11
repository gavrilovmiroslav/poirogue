
#include "config.h"
#include "common.h"
#include "engine.h"
#include "utils.h"
#include "graphs.h"
#include "level.h"
#include "ai.h"
#include "player.h"
#include "time.h"
#include "cursor.h"
#include "symbols.h"
#include "debug.h"
#include "hud.h"
#include "interactions.h"
#include "command_interp.h"

#undef main

struct BlockMovementThroughPeopleSystem
    : public OneOffSystem
    , public AccessEvents_Listen<CommandSignal>
    , public AccessWorld_UseUnique<CommandContext>
    , public AccessWorld_QueryAllEntitiesWith<Person, Name, WorldPosition>
    , public AccessWorld_QueryComponent<Player>
{
    void react_to_event(CommandSignal& signal)
    {
        if (signal.type != CommandType::Move) return;
        auto& context = AccessWorld_UseUnique<CommandContext>::access_unique();

        printf("[BLOCK MOV] Collision testing stared...\n");
        printf("Subject: #%d\n", context.subject);
        printf("Is player? #%s\n", AccessWorld_QueryComponent<Player>::has_component(context.subject) ? "yes" : "no");
        printf("Source: %d, %d\n", signal.data.move.from_x, signal.data.move.from_y);
        printf("Destination: %d, %d\n", signal.data.move.to_x, signal.data.move.to_y);

        for (auto&& [e, p, n, wp] : AccessWorld_QueryAllEntitiesWith<Person, Name, WorldPosition>::query().each())
        {
            printf("\t%s (%d) at position %d, %d\n", n.name.c_str(), p.person_id, wp.x, wp.y);
            if (context.subject == e)
            {
                printf("\t -- ignoring self\n");
                continue;
            }

            if (signal.data.move.to_x == wp.x && signal.data.move.to_y == wp.y)
            {
                printf("\t -- is blocking, will cancel\n");
                context.cancelled = true;
                break;
            }
        }

        printf("[BLOCK MOV] Testing complete.\n");
    }
};

int main(int argc, char* argv[])
{
    PoirogueEngine engine;
    auto level_creation = engine.add_one_off_system<LevelCreationSystem>();
    level_creation->social_interactions.push_back(murder_old_grievance_revenge);
    level_creation->social_interactions.push_back(murder_debt_scare);

    engine.add_one_off_system<PlayerCreationSystem>();
    engine.add_one_off_system<Debug_ReloadConfigSystem>();
    engine.add_one_off_system<TimeSystem>();

    engine.add_one_off_system<BlockMovementThroughPeopleSystem>();

    auto interp = engine.add_one_off_system<CommandInterpretationSystem>();
    interp->add_interpreter<CommandType::Wait>(new WaitCommandInterpreter);    
    interp->add_interpreter<CommandType::Move>(new MoveCommandInterpreter);

    engine.add_runtime_system<LevelRenderSystem>();
    engine.add_runtime_system<SymbolRenderSystem>();
    engine.add_runtime_system<MouseCursorSystem>();
    engine.add_runtime_system<PlayerChoiceSystem>();
    engine.add_runtime_system<AIChoiceSystem>();
    engine.add_runtime_system<Debug_RoomLevelRenderSystem>();
    engine.add_runtime_system<Debug_TurnOrderSystem>();    
    engine.add_runtime_system<Debug_HintSystem>();
    engine.add_runtime_system<HUDSystem>();

    engine.restart_game();
    
    while (engine) {
        engine.start_frame();
        engine.poll_events();
        engine.run_systems();
        engine.end_frame();
    }
}
