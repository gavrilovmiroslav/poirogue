
#include "time.h"
#include "config.h"
#include "common.h"

void TimeSystem::activate()
{
	auto& cal = AccessWorld_UseUnique<Calendar>::access_unique();
	cal.day = 1;
	cal.hour = 7;
	cal.minute = 0;

	ActionCompleteSignal signal{ 0 };
	react_to_event(signal);
}

void TimeSystem::react_to_event(ActionCompleteSignal& signal)
{
	auto& q = AccessWorld_UseUnique<TurnOrderQueue>::access_unique();
	auto& current_in_order = AccessWorld_UseUnique<CurrentInTurn>::access_unique();
	auto current_entity = current_in_order.current;

	if (!AccessWorld_QueryComponent<ActionPoints>::has_component(current_entity)
		|| !AccessWorld_QueryComponent<Speed>::has_component(current_entity))
	{
		current_entity = entt::null;
		while (!q.order.empty())
		{
			q.order.pop();
		}
	}

	if (current_entity != entt::null)
	{
		auto& points = AccessWorld_QueryComponent<ActionPoints>::get_component(current_entity);
		auto& speed = AccessWorld_QueryComponent<Speed>::get_component(current_entity);

		points.ap -= signal.cost;

		if (points.ap > 0)
		{			
			q.order.emplace(std::tuple<ActionPoints, Speed, Entity>(points, speed, current_entity));
		}
	}

	if (q.order.empty())
	{
		for (auto&& [e, ap, s] : AccessWorld_QueryAllEntitiesWith<ActionPoints, Speed>::query().each())
		{
			ap.ap += ACTION_POINTS_PER_TURN + (AccessWorld_QueryComponent<Player>::has_component(e) ? ACTION_POINTS_PLAYER_BONUS : 0);
			q.order.emplace(std::tuple<ActionPoints, Speed, Entity>{ ap, s, e });
		}

		AccessEvents_Emit<CalendarUpdateSignal>::emit_event();
	}

	const auto top = q.order.top();
	current_in_order.current = std::get<2>(top);
	q.order.pop();

	AccessEvents_Emit<AwaitingActionSignal>::emit_event(AwaitingActionSignal{ current_in_order.current });
}

void TimeSystem::react_to_event(CalendarUpdateSignal&)
{
	auto& cal = AccessWorld_UseUnique<Calendar>::access_unique();

	cal.minute++;
	if (cal.minute > 60)
	{
		cal.minute = 0;
		cal.hour++;

		AccessEvents_Emit<HourPassedSignal>::emit_event();

		if (cal.hour > 24)
		{
			cal.hour = 0;
			cal.day++;

			AccessEvents_Emit<DayPassedSignal>::emit_event();
		}
	}
}