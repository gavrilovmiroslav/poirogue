#pragma once

#include "graphs.h"
#include "commands.h"

struct Level;
struct LevelCreationEvent;
struct Player;
struct Health;

struct AIChoiceSystem
    : public RuntimeSystem    
    , public AccessWorld_QueryComponent<AIPlayer>    
    , public AccessWorld_QueryComponent<WorldPosition>
    , public AccessEvents_Listen<AwaitingActionSignal>
    , public AccessEvents_Emit<IssueCommandSignal>
{
    void react_to_event(AwaitingActionSignal& signal) override;

    void issue_command(IssueCommandSignal);
};