#include "command_interp.h"
#include "level.h"

void WaitCommandInterpreter::interpret_command(CommandContext& context, CommandSignal& signal)
{
	finish_command();
}

void MoveCommandInterpreter::interpret_command(CommandContext& context, CommandSignal& signal)
{
	const auto& level = AccessWorld_UseUnique<Level>::access_unique();

	const auto& speed = AccessWorld_QueryComponent<Speed>::get_component(context.subject);
	const auto cost_reduction = (speed.speed / ATTRIBUTE_SPEED_NORM) - 1;
	context.cost -= cost_reduction;

	if (level.map->isWalkable(signal.data.move.to_x, signal.data.move.to_y))
	{
		auto& world_pos = AccessWorld_QueryComponent<WorldPosition>::get_component(context.subject);
		world_pos.x = signal.data.move.to_x;
		world_pos.y = signal.data.move.to_y;

		if (AccessWorld_QueryComponent<Player>::has_component(context.subject))
		{
			const auto& sight = AccessWorld_QueryComponent<Sight>::get_component(context.subject);
			level.map->computeFov(world_pos.x, world_pos.y, sight.radius, true, FOV_RESTRICTIVE);
		}

        finish_command(context.cost);
	}
    else
    {
        finish_command(8);
    }	
}


void CommandInterpretationSystem::start_interpreting(IssueCommandSignal signal)
{
    auto& command_context = AccessWorld_UseUnique<CommandContext>::access_unique();
    command_context.cancelled = false;
    command_context.subject = signal.subject;
    command_context.targets.clear();
    for (auto target : signal.targets)
        command_context.targets.push_back(target);
    command_context.cost = ACTION_POINTS_PER_TURN;

    AccessEvents_Emit<CommandSignal>::emit_event(CommandSignal{ signal.type, signal.data });
}

void CommandInterpretationSystem::react_to_event(IssueCommandSignal& signal)
{
    if (current_command != nullptr)
    {
        issued_commands.push(signal);
    }
    else
    {
        start_interpreting(signal);
    }
}

void CommandInterpretationSystem::react_to_event(CommandSignal& signal)
{
    auto& context = AccessWorld_UseUnique<CommandContext>::access_unique();

    if (context.cancelled) return;

    if (interpreters.find(signal.type) != interpreters.end())
        interpreters[signal.type]->interpret_command(context, signal);
}

void CommandInterpretationSystem::react_to_event(CommandCompletedSignal&)
{
    if (issued_commands.size() > 0)
    {
        auto signal = issued_commands.front();
        issued_commands.pop();

        start_interpreting(signal);
    }
}

void CommandInterpretationSystem::react_to_event(CommandCancelledSignal&)
{
    if (issued_commands.size() > 0)
    {
        auto signal = issued_commands.front();
        issued_commands.pop();

        start_interpreting(signal);
    }
}