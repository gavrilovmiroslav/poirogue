#pragma once

#include "common.h"
#include "engine.h"

struct HUDSystem
	: public RuntimeSystem
	, public AccessConsole
	, public AccessResource_Mouse
	, public AccessWorld_UseUnique<Calendar>
	, public AccessWorld_UseUnique<GameContext>
	, public AccessWorld_QueryAllEntitiesWith<Player, Inventory>
	, public AccessWorld_QueryComponent<Item>
	, public AccessWorld_QueryComponent<Symbol>
{
	void label(std::string lab, std::string message, int x, int y, RGB label_color = "#ffffff"_rgb, RGB text_color = "#777777"_rgb);
	bool item_box(int index, Entity item, int x, int y = 3);
	void activate() override;
};