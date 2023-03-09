#include "ai.h"

void AIChoiceSystem::react_to_event(AwaitingActionSignal& signal)
{
	auto candidate = signal.current_in_order;
	if (AccessWorld_QueryComponent<AIPlayer>::has_component(candidate))
	{
		IssueCommandSignal issue;

		TCODRandom* rng = TCODRandom::getInstance();

		if (rng->getInt(0, 100) > 30)
		{
			issue.subject = candidate;
			issue.type = CommandType::Move;
			const auto& position = AccessWorld_QueryComponent<WorldPosition>::get_component(candidate);
			issue.data.move.from_x = position.x;
			issue.data.move.from_y = position.y;
			issue.data.move.to_x = position.x + rng->getInt(-1, 1);
			issue.data.move.to_y = position.y + rng->getInt(-1, 1);
			issue_command(issue);
		}
		else
		{
			issue.subject = candidate;
			issue.type = CommandType::Wait;
			issue_command(issue);
		}
	}	
}

void AIChoiceSystem::issue_command(IssueCommandSignal issue)
{
	AccessEvents_Emit<IssueCommandSignal>::emit_event(issue);
}