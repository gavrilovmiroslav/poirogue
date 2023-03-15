#pragma once

#include "common.h"
#include "engine.h"

struct TimeSystem
	: public OneOffSystem
	, public AccessWorld_UseUnique<Calendar>
	, public AccessWorld_UseUnique<TurnOrderQueue>
	, public AccessWorld_UseUnique<CurrentInTurn>
	, public AccessWorld_QueryAllEntitiesWith<ActionPoints, Speed>
	, public AccessWorld_QueryComponent<ActionPoints>
	, public AccessWorld_QueryComponent<Player>
	, public AccessWorld_QueryComponent<Speed>
	, public AccessWorld_QueryComponent<Person>
	, public AccessEvents_Emit<AwaitingActionSignal>
	, public AccessEvents_Emit<CalendarUpdateSignal>
	, public AccessEvents_Emit<HourPassedSignal>
	, public AccessEvents_Emit<DayPassedSignal>
	, public AccessEvents_Listen<ActionCompleteSignal>
	, public AccessEvents_Listen<CalendarUpdateSignal>
{
	bool pick_next_in_turn_order = false;

	void activate() override;
	void react_to_event(ActionCompleteSignal& signal) override;
	void react_to_event(CalendarUpdateSignal&) override;
};
