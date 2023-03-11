
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
    , public AccessWorld_UseUnique<Level>
    , public AccessWorld_UseUnique<CommandContext>
{
    void react_to_event(CommandSignal& signal)
    {
        auto& level = AccessWorld_UseUnique<Level>::access_unique();
        auto& context = AccessWorld_UseUnique<CommandContext>::access_unique();

        if (context.cancelled) return;

//        level.
//        if (interpreters.find(signal.type) != interpreters.end())
//            interpreters[signal.type]->interpret_command(context, signal);
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
