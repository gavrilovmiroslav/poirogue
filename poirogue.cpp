
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

#include "command_interp.h"

#undef main

int main(int argc, char* argv[])
{
    PoirogueEngine engine;
    engine.add_one_off_system<LevelCreationSystem>();
    engine.add_one_off_system<PlayerCreationSystem>();
    engine.add_one_off_system<Debug_ReloadConfigSystem>();
    engine.add_one_off_system<TimeSystem>();

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

    engine.restart_game();
    
    while (engine) {
        engine.start_frame();
        engine.poll_events();
        engine.run_systems();
        engine.end_frame();
    }
}
