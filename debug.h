#pragma once

#include "common.h"
#include "engine.h"

struct Debug_TurnOrderSystem
	: public RuntimeSystem
	, public AccessConsole
	, public AccessWorld_CheckValidity
	, public AccessWorld_UseUnique<CurrentInTurn>
	, public AccessWorld_QueryAllEntitiesWith<ActionPoints, Speed>
	, public AccessWorld_QueryComponent<Name>
{
	void activate() override;
};