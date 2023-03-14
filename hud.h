#pragma once

#include "common.h"
#include "engine.h"

struct HUDSystem
	: public RuntimeSystem
	, public AccessConsole
	, public AccessMousePosition
	, public AccessWorld_UseUnique<Calendar>
	, public AccessWorld_QueryAllEntitiesWith<Player, Inventory>
	, public AccessWorld_QueryComponent<Item>
	, public AccessWorld_QueryComponent<Symbol>
{
	void item_box(int index, Entity item, int x, int y = 2);
	void activate() override;
};