#pragma once

#include "common.h"
#include "engine.h"

struct Debug_TurnOrderSystem
	: public RuntimeSystem
	, public AccessConsole
	, public AccessWorld_CheckValidity
	, public AccessEvents_Listen<KeyEvent>
	, public AccessWorld_UseUnique<CurrentInTurn>
	, public AccessWorld_QueryAllEntitiesWith<ActionPoints, Speed, Health>
	, public AccessWorld_QueryComponent<Name>
{
	bool visible = false;

	void react_to_event(KeyEvent& signal) override;
	void activate() override;
};

struct Debug_ReloadConfigSystem
	: public OneOffSystem
	, public AccessYAML
	, public AccessEvents_Listen<KeyEvent>
	, public AccessWorld_UseUnique<Colors>
{
	void react_to_event(KeyEvent& signal) override;	
};

struct Debug_HintSystem
	: public RuntimeSystem
	, public AccessConsole
{
	void activate() override;
};