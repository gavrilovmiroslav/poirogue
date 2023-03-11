#pragma once

#include "common.h"
#include "engine.h"
#include "commands.h"

#include <queue>

struct CommandInterpreter
    : public AccessEvents_Emit<ActionCompleteSignal>
    , public AccessEvents_Emit<CommandCompletedSignal>
{
    void finish_command(int cost = ACTION_POINTS_PER_TURN)
    {
        AccessEvents_Emit<CommandCompletedSignal>::emit_event(CommandCompletedSignal{});
        AccessEvents_Emit<ActionCompleteSignal>::emit_event(ActionCompleteSignal{ cost });
    }
    
    virtual void interpret_command(CommandContext&, CommandSignal&) = 0;
};

struct CommandInterpretationSystem
    : public OneOffSystem
    , public AccessEvents_Emit<ActionCompleteSignal>
    , public AccessEvents_Listen<IssueCommandSignal>
    , public AccessEvents_Listen<CommandSignal>
    , public AccessEvents_Listen<CommandCompletedSignal>
    , public AccessEvents_Listen<CommandCancelledSignal>
    , public AccessEvents_Emit<CommandSignal>
    , public AccessEvents_Emit<CommandCompletedSignal>
    , public AccessEvents_Emit<CommandCancelledSignal>    
    , public AccessWorld_UseUnique<CommandContext>
{
    std::queue<IssueCommandSignal> issued_commands;
    IssueCommandSignal* current_command = nullptr;

    void start_interpreting(IssueCommandSignal signal);

    void react_to_event(IssueCommandSignal& signal) override;
    void react_to_event(CommandSignal& signal) override;
    void react_to_event(CommandCompletedSignal&) override;
    void react_to_event(CommandCancelledSignal&) override;

    template<CommandType C>
    void add_interpreter(CommandInterpreter* c)
    {        
        interpreters.insert({ C, c });
    }

private:
    std::unordered_map<CommandType, CommandInterpreter*> interpreters;
};


struct WaitCommandInterpreter
    : public CommandInterpreter
{
    void interpret_command(CommandContext&, CommandSignal&) override;
};

struct Level;

struct MoveCommandInterpreter
    : public CommandInterpreter
    , public AccessWorld_QueryComponent<WorldPosition>
    , public AccessWorld_QueryComponent<Speed>
    , public AccessWorld_QueryComponent<Player>
    , public AccessWorld_QueryComponent<Sight>
    , public AccessWorld_UseUnique<Level>
{
    void interpret_command(CommandContext&, CommandSignal&) override;
};